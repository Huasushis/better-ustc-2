use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce
};
use tauri::AppHandle;
use sha2::{Sha256, Digest};
use tauri_plugin_machine_uid::MachineUidExt;
use rand::{RngCore, thread_rng};
use anyhow::{Result, Context};

/// 获取基于机器特征的固定 32 字节密钥
fn get_machine_key(app: &AppHandle) -> Result<[u8; 32]> {
    // 获取机器唯一 ID (如果获取失败，使用一个固定的 fallback，虽然安全性降低但保证可用性)
    let machine_id = app.machine_uid().get_machine_uid().unwrap().id.unwrap_or_else(|| "IDONTKNOWWHATTOWRITEDOWNHERE".to_string());
    
    // 加上一个硬编码的 Salt，防止彩虹表
    let salt = "BETTER_USTC_SALT_2025";
    let combined = format!("{}{}", machine_id, salt);

    // 使用 SHA256 生成 32 字节的 Key
    let mut hasher = Sha256::new();
    hasher.update(combined.as_bytes());
    let result = hasher.finalize();
    
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    Ok(key)
}

/// 加密字符串
/// 返回格式: hex(nonce) + ":" + hex(ciphertext)
pub fn encrypt_data(app: &AppHandle, plaintext: &str) -> Result<String> {
    let key_bytes = get_machine_key(app)?;
    let cipher = Aes256Gcm::new(&key_bytes.into());

    // 生成随机 Nonce (12 bytes for GCM)
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

    let nonce_hex = hex::encode(nonce_bytes);
    let cipher_hex = hex::encode(ciphertext);

    Ok(format!("{}:{}", nonce_hex, cipher_hex))
}

/// 解密字符串
pub fn decrypt_data(app: &AppHandle, encrypted_str: &str) -> Result<String> {
    let parts: Vec<&str> = encrypted_str.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid encrypted format");
    }

    let nonce_bytes = hex::decode(parts[0]).context("Failed to decode nonce")?;
    let ciphertext_bytes = hex::decode(parts[1]).context("Failed to decode ciphertext")?;

    if nonce_bytes.len() != 12 {
        anyhow::bail!("Invalid nonce length");
    }

    let key_bytes = get_machine_key(app)?;
    let cipher = Aes256Gcm::new(&key_bytes.into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext_bytes.as_ref())
        .map_err(|e| anyhow::anyhow!("Decryption failed (Wrong machine?): {}", e))?;

    let plaintext = String::from_utf8(plaintext_bytes)?;
    Ok(plaintext)
}