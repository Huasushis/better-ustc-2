use std::env;
use std::sync::Arc;

use tauri_plugin_http::reqwest::Client;

use aes::cipher::{generic_array::GenericArray, BlockEncryptMut, KeyInit};
use aes::Aes128;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use ecb::Encryptor;
use regex::Regex;
use serde_json::Value;

use reqwest_cookie_store::CookieStoreMutex;

use crate::rustustc::cas::info::UserInfo;
use crate::rustustc::url::generate_url;

pub struct CASClient {
    client: Client,
    pub cookie_store: Arc<CookieStoreMutex>,
}

impl CASClient {
    pub fn new() -> Self {
        let cookie_store = Arc::new(CookieStoreMutex::default());

        let client = Client::builder()
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();

        Self {
            client,
            cookie_store,
        }
    }

    pub fn client_ref(&self) -> &Client {
        &self.client
    }

    fn aes_encrypt(data: &str, key_base64: &str) -> Result<String> {
        let key = general_purpose::STANDARD
            .decode(key_base64)
            .context("Invalid base64 key")?;
        if key.len() != 16 {
            bail!("Key length expected 16, got {}", key.len());
        }

        let block_size = 16;
        let mut buffer = data.as_bytes().to_vec();
        let padding_len = block_size - (buffer.len() % block_size);
        let padding_byte = padding_len as u8;
        buffer.resize(buffer.len() + padding_len, padding_byte);

        let mut encryptor = Encryptor::<Aes128>::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Init aes error: {}", e))?;

        let mut encrypted_data = Vec::new();
        for chunk in buffer.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            encryptor.encrypt_block_mut(&mut block);
            encrypted_data.extend_from_slice(&block);
        }
        Ok(general_purpose::STANDARD.encode(encrypted_data))
    }

    fn get_usr_and_pwd(
        &self,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<(String, String)> {
        let u = match username {
            Some(s) => s.to_string(),
            None => env::var("USTC_CAS_USR").context("USTC_CAS_USR not set")?,
        };
        let p = match password {
            Some(s) => s.to_string(),
            None => env::var("USTC_CAS_PWD").context("USTC_CAS_PWD not set")?,
        };
        Ok((u, p))
    }

    pub async fn login_by_pwd(&self, username: Option<&str>, password: Option<&str>) -> Result<()> {
        let (usr, pwd) = self.get_usr_and_pwd(username, password)?;
        let login_url = generate_url("id", "cas/login");

        let resp = self.client.get(&login_url).send().await?.text().await?;

        let re_crypto = Regex::new(r#"<p id="login-croypto">(.+)</p>"#)?;
        let re_flow = Regex::new(r#"<p id="login-page-flowkey">(.+)</p>"#)?;

        let crypto = re_crypto
            .captures(&resp)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .context("Missing crypto")?
            .to_string();
        let flow_key = re_flow
            .captures(&resp)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str())
            .context("Missing flowkey")?
            .to_string();

        let enc_pwd = Self::aes_encrypt(&pwd, &crypto)?;
        let enc_captcha = Self::aes_encrypt("{}", &crypto)?;

        let params = [
            ("type", "UsernamePassword"),
            ("_eventId", "submit"),
            ("croypto", &crypto),
            ("username", &usr),
            ("password", &enc_pwd),
            ("captcha_payload", &enc_captcha),
            ("execution", &flow_key),
        ];

        let res = self.client.post(&login_url).form(&params).send().await?;

        let final_url = res.url().as_str();
        let status = res.status();

        if !final_url.contains("cas/login") {
            Ok(())
        } else {
            let text = res.text().await?;
            let re_err = Regex::new(
                r#"<div\s+class="alert alert-danger"\s+id="login-error-msg">\s*<span>([^<]+)</span>\s*</div>"#,
            )?;
            if let Some(caps) = re_err.captures(&text) {
                bail!("Login failed: {}", &caps[1]);
            }
            let debug_text = if text.len() > 200 {
                &text[..200]
            } else {
                &text
            };
            bail!(
                "Login failed: Unknown error (status code {}). Body start: {}",
                status,
                debug_text
            );
        }
    }

    pub async fn is_login(&self) -> bool {
        let url = generate_url("id", "cas/login");
        let res = self.client.get(&url).send().await;
        match res {
            Ok(resp) => {
                let final_url = resp.url().as_str();
                !final_url.contains("cas/login")
            }
            Err(_) => false,
        }
    }

    pub async fn logout(&self) -> Result<()> {
        let logout_url = generate_url("id", "gate/logout");
        let res = self.client.get(&logout_url).send().await;
        match res {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Logout failed: {}", e)),
        }
    }

    pub async fn get_info(&self) -> Result<UserInfo> {
        let user_url = generate_url("id", "gate/getUser");

        let resp = self.client.get(&user_url).send().await?;

        if !resp.status().is_success() {
            bail!(
                "get_info failed. Status: {}. Maybe cookie expired?",
                resp.status()
            );
        }

        let user_resp_text = resp.text().await?;
        if user_resp_text.trim().is_empty() {
            bail!("get_info returned empty body.");
        }

        let user_resp: Value =
            serde_json::from_str(&user_resp_text).context("Failed to parse getUser JSON")?;

        let object_id = user_resp["objectId"]
            .as_str()
            .context("Failed to get object_id")?;
        let username = user_resp["username"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        let pid_path = format!("gate/linkid/api/user/getPersonId/{}", object_id);
        let pid_url = generate_url("id", &pid_path);
        let pid_resp_text = self.client.get(&pid_url).send().await?.text().await?;
        let pid_resp: Value =
            serde_json::from_str(&pid_resp_text).context("Failed to parse getPersonId JSON")?;

        let person_id = pid_resp["data"]
            .as_str()
            .context("Failed to get person_id")?;

        let info_path = format!("gate/linkid/api/aggregate/user/userInfo/{}", person_id);
        let info_url = generate_url("id", &info_path);

        let info_resp_text = self.client.post(&info_url).send().await?.text().await?;
        let info_resp: Value =
            serde_json::from_str(&info_resp_text).context("Failed to parse userInfo JSON")?;

        let info_data = info_resp["data"].clone();

        Ok(UserInfo::new(username, info_data))
    }
}
