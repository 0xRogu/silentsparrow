use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug)]
pub struct Canary {
    pub timestamp: String,

    pub message: String,

    pub signature: String,

    pub log_hash: String,
}

impl Canary {
    pub fn new(message: &str, signature: &[u8], log_data: Option<&[u8]>) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        let log_hash = match log_data {
            Some(data) => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
            None => String::from("none"),
        };
        Canary {
            timestamp,
            message: message.to_string(),
            signature: hex::encode(signature),
            log_hash,
        }
    }
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
