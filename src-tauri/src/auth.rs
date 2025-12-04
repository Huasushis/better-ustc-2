use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_store::StoreExt;
use serde_json::json;
use anyhow::{Result, Context};

use crate::state::AppState;
use crate::rustustc::cas::client::CASClient;
use crate::rustustc::young::YouthService;
use crate::rustustc::young::model::User;
use crate::security::{encrypt_data, decrypt_data};

const CREDENTIALS_STORE: &str = "credentials.json";

/// 执行核心登录逻辑：CAS Login -> Youth Service Init -> Update State
pub async fn perform_login(
    state: &State<'_, AppState>,
    username: &str,
    password: &str
) -> Result<User> {
    // 1. CAS 登录
    let client = CASClient::new();
    client.login_by_pwd(Some(username), Some(password)).await?;

    // 2. Youth Service 初始化
    let youth = YouthService::new(&client).await?;
    
    let client_arc = Arc::new(client);
    let youth_arc = Arc::new(youth);

    // 3. 获取用户信息 (验证 Token 有效性)
    let user_info = User::get_current(&youth_arc).await?;

    // 4. 更新全局状态
    *state.cas_client.lock().await = Some(client_arc);
    *state.youth_service.lock().await = Some(youth_arc);

    Ok(user_info)
}

/// 保存凭据 (加密)
pub fn save_credentials(app: &AppHandle, username: &str, password: &str) -> Result<()> {
    let store = app.store(CREDENTIALS_STORE).context("Failed to access store")?;
    
    // 加密密码
    let encrypted_pwd = encrypt_data(app, password)?;
    
    store.set("username", json!(username));
    store.set("password", json!(encrypted_pwd)); // 存储密文
    store.save().context("Failed to save store")?;
    Ok(())
}

/// 尝试自动登录
/// 返回: (是否登录成功, 是否有存储的账号, 用户名, 用户信息/错误信息)
pub async fn try_auto_login(
    app: &AppHandle,
    state: &State<'_, AppState>
) -> Result<(bool, bool, Option<String>, Option<User>)> {
    
    // 1. 检查内存状态
    {
        let youth_guard = state.youth_service.lock().await;
        if let Some(youth) = &*youth_guard {
            if let Ok(info) = User::get_current(youth).await {
                return Ok((true, true, Some(info.id.clone()), Some(info)));
            }
        }
    }

    // 2. 检查磁盘存储
    let store = app.store(CREDENTIALS_STORE).context("Failed to access store")?;
    
    let username_opt = store.get("username").and_then(|v| v.as_str().map(|s| s.to_string()));
    let encrypted_pwd_opt = store.get("password").and_then(|v| v.as_str().map(|s| s.to_string()));

    if let (Some(ref username), Some(ref encrypted_pwd)) = (&username_opt, &encrypted_pwd_opt) {
        // 解密密码
        let password = match decrypt_data(app, &encrypted_pwd) {
            Ok(p) => p,
            Err(e) => {
                // 解密失败（可能换了机器或文件损坏），视为无凭据，或者返回错误
                println!("Decryption failed: {}", e);
                return Ok((false, true, Some(username.clone()), None));
            }
        };

        // 尝试登录
        match perform_login(state, username, &password).await {
            Ok(user) => Ok((true, true, Some(username.clone()), Some(user))),
            Err(_) => Ok((false, true, Some(username.clone()), None)), // 登录失败但账号存在
        }
    } else {
        if let Some(username) = username_opt {
            // 只有用户名，没有密码
            Ok((false, true, Some(username.clone()), None))
        } else {
            // 完全没有凭据
            Ok((false, false, None, None))
        }
    }
}

/// 登出并清除凭据
pub fn clear_credentials(app: &AppHandle) -> Result<()> {
    let store = app.store(CREDENTIALS_STORE).context("Failed to access store")?;
    store.delete("password"); // 只删密码
    store.save().context("Failed to save store")?;
    Ok(())
}