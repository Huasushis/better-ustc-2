use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(default)]
    pub id: String,
    #[serde(rename = "XM")]
    pub name: String,
    #[serde(rename = "GID")]
    pub gid: String,
    #[serde(rename = "MBEMAIL")]
    pub email: Option<String>,
}

impl UserInfo {
    pub fn new(id: String, data: serde_json::Value) -> Self {
        let mut info: UserInfo =
            serde_json::from_value(data).expect("Failed to parse user info data");
        info.id = id;
        info
    }
}
