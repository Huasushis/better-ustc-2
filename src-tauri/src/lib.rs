// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod auth;
pub mod recommend;
pub mod rustustc;
pub mod security;
pub mod state;

use crate::recommend::Recommender;
use crate::rustustc::young::{SCFilter, SecondClass, YouthService};
use crate::state::AppState;
use serde_json::json;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_machine_uid::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            login,
            logout,
            get_login_status,
            refresh_session,
            get_unended_activities,
            get_registered_activities,
            get_participated_activities,
            register_for_activity,
            get_recommended_activities,
            get_activity_children,
            get_activity_detail,
            get_class_schedule,
            get_pending_appeals
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn map_err<E: ToString>(e: E) -> String {
    json!({ "code": "INTERNAL_ERROR", "message": e.to_string() }).to_string()
}

async fn get_service(state: &State<'_, AppState>) -> Result<Arc<YouthService>, String> {
    let guard = state.youth_service.lock().await;
    match &*guard {
        Some(s) => Ok(s.clone()),
        None => Err(json!({"code": "AUTH_REQUIRED", "message": "Please login first"}).to_string()),
    }
}

// ==================== 登录相关 (调用 auth 模块) ====================

/// CAS + 二课登录。
/// - `save=true` 时将账号/密文密码写入插件存储，密钥来源 machine_uid。
/// - 错误统一包装为 `{code,message}` JSON 字符串，前端需先尝试 JSON.parse。
#[tauri::command]
async fn login(
    app: AppHandle,
    state: State<'_, AppState>,
    username: String,
    password: String,
    save: bool,
) -> Result<serde_json::Value, String> {
    // 1. 执行登录
    let user_info = auth::perform_login(&state, &username, &password)
        .await
        .map_err(map_err)?;

    // 2. 保存加密凭据
    if save {
        auth::save_credentials(&app, &username, &password).map_err(map_err)?;
    }

    Ok(json!(user_info))
}

/// 查询登录状态：优先复用内存会话，否则尝试读取磁盘密文自动登录。
#[tauri::command]
async fn get_login_status(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let (logged_in, has_creds, username, user) =
        auth::try_auto_login(&app, &state).await.map_err(map_err)?;

    if logged_in {
        Ok(json!({
            "logged_in": true,
            "user": user
        }))
    } else {
        Ok(json!({
            "logged_in": false,
            "has_stored_creds": has_creds,
            "username": username
        }))
    }
}

/// 清空会话并删除存储的密码（用户名保留，便于自动填充）。
#[tauri::command]
async fn logout(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    *state.cas_client.lock().await = None;
    *state.youth_service.lock().await = None;
    auth::clear_credentials(&app).map_err(map_err)?;
    Ok(())
}

/// 使用当前 CAS Cookie 刷新 YouthService。若 Cookie 失效会返回错误 JSON。
#[tauri::command]
async fn refresh_session(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let cas_guard = state.cas_client.lock().await;
    let cas_client = match &*cas_guard {
        Some(c) => c.clone(),
        None => return Err(map_err("Session expired. Please login again.")),
    };
    drop(cas_guard);

    if !cas_client.is_login().await {
        return Err(map_err("CAS Session expired. Please login again."));
    }

    let new_youth = YouthService::new(&cas_client)
        .await
        .map_err(|e| map_err(format!("Failed to refresh: {}", e)))?;
    let youth_arc = Arc::new(new_youth);

    *state.youth_service.lock().await = Some(youth_arc.clone());

    let user_info = crate::rustustc::young::model::User::get_current(&youth_arc)
        .await
        .map_err(map_err)?;

    Ok(json!({ "success": true, "user": user_info }))
}

// ==================== 二课活动相关 (修改筛选逻辑) ====================

/// 获取未结束的活动列表（不展开系列课）。
#[tauri::command]
async fn get_unended_activities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let activities = SecondClass::find(&service, SCFilter::new(), false, false, -1)
        .await
        .map_err(map_err)?;
    Ok(json!(activities))
}

/// 获取已报名 / 报名已结束的活动。
#[tauri::command]
async fn get_registered_activities(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;

    let all_my_activities = SecondClass::get_participated(&service)
        .await
        .map_err(map_err)?;

    let registered_activities: Vec<&SecondClass> = all_my_activities
        .iter()
        .filter(|sc| {
            use crate::rustustc::young::model::Status;
            matches!(sc.status(), Status::Applying | Status::ApplyEnded)
        })
        .collect();

    Ok(json!(registered_activities))
}

/// 获取已参与/已结项的活动。
#[tauri::command]
async fn get_participated_activities(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let all = SecondClass::get_participated(&service)
        .await
        .map_err(map_err)?;

    // 过滤：已完成的活动
    let finished: Vec<&SecondClass> = all
        .iter()
        .filter(|sc| {
            use crate::rustustc::young::model::Status;
            !matches!(sc.status(), Status::Applying | Status::ApplyEnded)
        })
        .collect();

    Ok(json!(finished))
}

/// 报名指定活动；会先更新详情再 apply，必要时自动取消冲突活动后重试。
#[tauri::command]
async fn register_for_activity(
    state: State<'_, AppState>,
    activity_id: String,
) -> Result<bool, String> {
    let service = get_service(&state).await?;
    // 简单构造 dummy 对象以便调用 update -> apply
    let mut sc = SecondClass {
        id: activity_id,
        name: "".into(),
        status_code: 0,
        valid_hour: None,
        apply_num: None,
        apply_limit: None,
        boolean_registration: None,
        need_sign_info_str: None,
        conceive: None,
        base_content: None,
        item_category: None,
        create_time_str: None,
        apply_start: None,
        apply_end: None,
        start_time: None,
        end_time: None,
        tel: None,
        raw: serde_json::Value::Null,
    };
    sc.update(&service).await.map_err(map_err)?;
    sc.apply(&service, false, true, None).await.map_err(map_err)
}

/// 基于历史参与记录的简单推荐（TF/部门/模块加权）。
#[tauri::command]
async fn get_recommended_activities(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let rec_list = Recommender::recommend(&service, 10)
        .await
        .map_err(map_err)?;
    Ok(json!(rec_list))
}

/// 获取系列课的子项目；非系列课返回 `NOT_A_SERIES` 错误。
#[tauri::command]
async fn get_activity_children(
    state: State<'_, AppState>,
    activity_id: String,
) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;

    // 1. 构造一个只有 ID 的对象
    let mut sc = SecondClass {
        id: activity_id,
        name: "".into(),
        status_code: 0,
        valid_hour: None,
        apply_num: None,
        apply_limit: None,
        boolean_registration: None,
        need_sign_info_str: None,
        conceive: None,
        base_content: None,
        item_category: None,
        create_time_str: None,
        apply_start: None,
        apply_end: None,
        start_time: None,
        end_time: None,
        tel: None,
        raw: serde_json::Value::Null,
    };

    // 2. 更新详情 (这一步是为了获取 is_series 标志，以及确保 ID 有效)
    sc.update(&service).await.map_err(map_err)?;

    if !sc.is_series() {
        return Err(json!({
            "code": "NOT_A_SERIES",
            "message": "The specified activity is not a series activity."
        })
        .to_string());
    }

    // 3. 获取子项目
    // 注意：get_children 内部会检查 is_series()，如果不是系列课会返回空列表
    let children = sc.get_children(&service).await.map_err(map_err)?;

    Ok(json!(children))
}

#[tauri::command]
async fn get_activity_detail(
    state: State<'_, AppState>,
    activity_id: String,
) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;

    // 1. 构造 dummy 对象
    let mut sc = SecondClass {
        id: activity_id,
        name: "".into(),
        status_code: 0,
        valid_hour: None,
        apply_num: None,
        apply_limit: None,
        boolean_registration: None,
        need_sign_info_str: None,
        conceive: None,
        base_content: None,
        item_category: None,
        create_time_str: None,
        apply_start: None,
        apply_end: None,
        start_time: None,
        end_time: None,
        tel: None,
        raw: serde_json::Value::Null,
    };

    // 2. 调用 update 从服务器获取最新详情
    sc.update(&service).await.map_err(map_err)?;

    // 3. 返回完整的对象
    Ok(json!(sc))
}

//TODO

#[tauri::command]
async fn get_class_schedule() -> Result<serde_json::Value, String> {
    Ok(json!([]))
}

#[tauri::command]
async fn get_pending_appeals() -> Result<serde_json::Value, String> {
    Ok(json!([]))
}
