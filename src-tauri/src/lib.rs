// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod rustustc;
pub mod state;
pub mod recommend;
pub mod security;
pub mod auth;

use std::sync::Arc;
use tauri::{AppHandle, State};
use serde_json::json;
// use chrono::Local;
use crate::state::AppState;
use crate::rustustc::young::{YouthService, SecondClass, SCFilter};
use crate::recommend::Recommender;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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

#[tauri::command]
async fn get_login_status(
    app: AppHandle,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    let (logged_in, has_creds, username, user) = auth::try_auto_login(&app, &state)
        .await
        .map_err(map_err)?;

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

#[tauri::command]
async fn logout(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    *state.cas_client.lock().await = None;
    *state.youth_service.lock().await = None;
    auth::clear_credentials(&app).map_err(map_err)?;
    Ok(())
}

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

#[tauri::command]
async fn get_unended_activities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let activities = SecondClass::find(&service, SCFilter::new(), false, false, -1)
        .await
        .map_err(map_err)?;
    Ok(json!(activities))
}

#[tauri::command]
async fn get_registered_activities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    
    let all_my_activities = SecondClass::get_participated(&service)
        .await
        .map_err(map_err)?;
    
    let registered_activities: Vec<&SecondClass> = all_my_activities.iter().filter(|sc| {
        use crate::rustustc::young::model::Status;
        matches!(sc.status(), Status::Applying | Status::ApplyEnded)
    }).collect();

    Ok(json!(registered_activities))
}

#[tauri::command]
async fn get_participated_activities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let all = SecondClass::get_participated(&service).await.map_err(map_err)?;
    
    // 过滤：已完成的活动
    let finished: Vec<&SecondClass> = all.iter().filter(|sc| {
        use crate::rustustc::young::model::Status;
        !matches!(sc.status(), Status::Applying | Status::ApplyEnded)
    }).collect();
    
    Ok(json!(finished))
}

#[tauri::command]
async fn register_for_activity(state: State<'_, AppState>, activity_id: String) -> Result<bool, String> {
    let service = get_service(&state).await?;
    // 简单构造 dummy 对象以便调用 update -> apply
    let mut sc = SecondClass {
        id: activity_id,
        name: "".into(), status_code: 0, valid_hour: None, apply_num: None, apply_limit: None,
        boolean_registration: None, need_sign_info_str: None, conceive: None, base_content: None, item_category: None,
        create_time_str: None, apply_start: None, apply_end: None, start_time: None, end_time: None,
        tel: None, raw: serde_json::Value::Null,
    };
    sc.update(&service).await.map_err(map_err)?;
    sc.apply(&service, false, true, None).await.map_err(map_err)
}

#[tauri::command]
async fn get_recommended_activities(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let service = get_service(&state).await?;
    let rec_list = Recommender::recommend(&service, 10).await.map_err(map_err)?;
    Ok(json!(rec_list))
}

#[tauri::command]
async fn get_class_schedule() -> Result<serde_json::Value, String> { Ok(json!([])) }

#[tauri::command]
async fn get_pending_appeals() -> Result<serde_json::Value, String> { Ok(json!([])) }