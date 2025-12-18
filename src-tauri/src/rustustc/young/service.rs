use aes::Aes128;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose, Engine as _};
use cbc::cipher::{BlockEncryptMut, KeyIvInit};
use cbc::Encryptor;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri_plugin_http::reqwest::Url;

use crate::rustustc::cas::client::CASClient;
use crate::rustustc::url::generate_url;

type Aes128CbcEnc = Encryptor<Aes128>;

pub struct YouthService {
    access_token: String,
    client: tauri_plugin_http::reqwest::Client,
    pub retry: u32,
}

impl YouthService {
    pub async fn new(cas_client: &CASClient) -> Result<Self> {
        // 1. 构造 CAS 登录 URL
        let service_url = generate_url("young", "login/sc-wisdom-group-learning/");
        let login_url_base = generate_url("id", "cas/login");
        let login_url = Url::parse_with_params(&login_url_base, &[("service", &service_url)])?;

        let resp = cas_client.client_ref().get(login_url).send().await?;
        let final_url = resp.url();
        let ticket = final_url
            .query_pairs()
            .find(|(k, _)| k == "ticket")
            .map(|(_, v)| v.to_string());

        let ticket = match ticket {
            Some(t) => t,
            None => bail!("Failed to get Service Ticket. You might not be logged in."),
        };

        let check_url = generate_url(
            "young",
            "login/wisdom-group-learning-bg/cas/client/checkSsoLogin",
        );
        let res_bytes = cas_client
            .client_ref()
            .get(&check_url)
            .query(&[("ticket", &ticket), ("service", &service_url)])
            .send()
            .await?
            .bytes()
            .await?;
        let res_text = String::from_utf8(res_bytes.to_vec())?;
        let res: Value =
            serde_json::from_str(&res_text).context("Failed to parse Youth login JSON")?;

        if !res["success"].as_bool().unwrap_or(false) {
            bail!("Youth login failed: {}", res["message"]);
        }

        let token = res["result"]["token"]
            .as_str()
            .context("Missing token")?
            .to_string();

        Ok(Self {
            access_token: token,
            client: cas_client.client_ref().clone(),
            retry: 3, // 默认重试3次
        })
    }

    fn encrypt(&self, data: &Value, timestamp: u64) -> Result<String> {
        // Token 末尾 32 字符被切分为 key+iv（各 16 字节）；若后端变更 token 长度，这里将 panic，建议未来显式校验长度并返回友好错误。
        let token_len = self.access_token.len();
        let key_start = token_len.saturating_sub(16);
        let key_str = &self.access_token[key_start..];
        let iv_start = token_len.saturating_sub(32);
        let iv_end = token_len.saturating_sub(16);
        let iv_str = &self.access_token[iv_start..iv_end];

        let key = key_str.as_bytes();
        let iv = iv_str.as_bytes();

        let mut combined = data.clone();
        if let Value::Object(ref mut map) = combined {
            map.insert("_t".to_string(), json!(timestamp));
        }
        let json_string = combined.to_string();
        let plaintext = json_string.as_bytes();

        let block_size = 16;
        let len = plaintext.len();
        let padding_len = block_size - (len % block_size);
        let mut buffer = plaintext.to_vec();
        for _ in 0..padding_len {
            buffer.push(padding_len as u8);
        }

        let mut encryptor = Aes128CbcEnc::new_from_slices(key, iv)
            .map_err(|e| anyhow::anyhow!("Invalid Key/IV for AES: {}", e))?;

        let mut blocks = Vec::new();
        for chunk in buffer.chunks_mut(16) {
            blocks.push(cbc::cipher::generic_array::GenericArray::from_mut_slice(
                chunk,
            ));
        }
        for block in blocks {
            encryptor.encrypt_block_mut(block);
        }
        Ok(general_purpose::STANDARD.encode(buffer))
    }

    pub async fn request(
        &self,
        endpoint: &str,
        method: &str,
        params: Option<Value>,
        json_body: Option<Value>,
    ) -> Result<Value> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let url = generate_url(
            "young",
            &format!("login/wisdom-group-learning-bg/{}", endpoint),
        );

        let encrypted_params = if let Some(p) = &params {
            self.encrypt(p, timestamp)?
        } else {
            self.encrypt(&json!({}), timestamp)?
        };

        let req = match method.to_lowercase().as_str() {
            "get" => self.client.get(&url).query(&[
                ("requestParams", &encrypted_params),
                ("_t", &timestamp.to_string()),
            ]),
            "post" => {
                let encrypted_body = if let Some(j) = &json_body {
                    self.encrypt(j, timestamp)?
                } else {
                    self.encrypt(&json!({}), timestamp)?
                };
                let body_val = json!({ "requestParams": encrypted_body });
                self.client
                    .post(&url)
                    .header("Content-Type", "application/json")
                    .body(body_val.to_string())
            }
            _ => bail!("Unsupported method"),
        };

        let resp = req
            .header("X-Access-Token", &self.access_token)
            .send()
            .await?;
        let resp_bytes = resp.bytes().await?;
        let resp_text = String::from_utf8(resp_bytes.to_vec())?;
        let resp_json: Value = serde_json::from_str(&resp_text)?;

        if resp_json["success"].as_bool().unwrap_or(false) {
            Ok(resp_json["result"].clone())
        } else {
            bail!("API Error: {}", resp_json["message"]);
        }
    }

    pub async fn get_result(&self, url: &str, params: Option<Value>) -> Result<Value> {
        let mut last_error = anyhow::anyhow!("Max retry reached");
        for _ in 0..self.retry {
            match self.request(url, "get", params.clone(), None).await {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = e;
                    continue;
                }
            }
        }
        Err(last_error)
    }

    pub async fn page_search(
        &self,
        url: &str,
        params: Value,
        max: i32,
        size: i32,
    ) -> Result<Vec<Value>> {
        let mut results = Vec::new();
        let mut page = 1;
        let mut params = params;
        let mut remaining = max;

        loop {
            if max != -1 && remaining <= 0 {
                break;
            }

            params["pageNo"] = json!(page);
            params["pageSize"] = json!(size);

            let res = self.get_result(url, Some(params.clone())).await?;

            let records = res["records"]
                .as_array()
                .context("Response missing records")?;
            let total = res["total"].as_u64().unwrap_or(0);

            for record in records {
                results.push(record.clone());
                if max != -1 {
                    remaining -= 1;
                    if remaining <= 0 {
                        break;
                    }
                }
            }

            if (page as u64 * size as u64) >= total {
                break;
            }
            page += 1;
        }
        Ok(results)
    }
}
