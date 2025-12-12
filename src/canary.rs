use crate::config::Config;
use crate::crypto::Crypto;
use crate::publisher::HttpsPublisher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct SparrowSong {
    pub timestamp: String,
    pub message: String,
    pub signature: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_hash: Option<String>,
}

pub struct Canary {
    config: Config,
    crypto: Crypto,
    publisher: Option<HttpsPublisher>,
    last_update: Option<DateTime<Utc>>,
}

impl Canary {
    pub fn new(config: Config) -> Self {
        let crypto = Crypto::load_or_create();
        let publisher = config
            .publish_url
            .as_ref()
            .map(|url| HttpsPublisher::new(url.clone(), config.publish_token.clone()));
        let last_update = Self::read_last_timestamp(&config.output_path);
        Self {
            config,
            crypto,
            publisher,
            last_update,
        }
    }

    fn read_last_timestamp(path: impl AsRef<Path>) -> Option<DateTime<Utc>> {
        let path = path.as_ref();
        if !path.exists() {
            return None;
        }
        let content = fs::read_to_string(path).ok()?;
        let song: SparrowSong = serde_json::from_str(&content).ok()?;
        DateTime::parse_from_rfc3339(&song.timestamp)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    pub async fn refresh(&mut self) -> Result<(), String> {
        let now = Utc::now();
        let message = if let Some(last) = self.last_update {
            let interval = chrono::Duration::from_std(self.config.interval_duration())
                .map_err(|e| format!("Duration conversion failed: {}", e))?;
            if now.signed_duration_since(last) > interval {
                self.config.message_overdue.clone()
            } else {
                self.config.message_normal.clone()
            }
        } else {
            self.config.message_normal.clone()
        };

        let payload = SparrowSong {
            timestamp: now.to_rfc3339(),
            message,
            signature: String::new(),
            public_key: self.crypto.public_key_hex(),
            log_hash: None,
        };

        let to_sign = format!(
            "{}\n{}\n{}",
            payload.timestamp, payload.message, payload.public_key,
        );

        let signature_bytes = self.crypto.sign(to_sign.as_bytes());
        let mut payload = payload;
        payload.signature = hex::encode(signature_bytes);

        let json = serde_json::to_string_pretty(&payload)
            .map_err(|e| format!("JSON serialization failed: {e}"))?;

        self.atomic_write_to_disk(&json)?;

        if let Some(publisher) = &self.publisher {
            if let Err(e) = publisher.publish(&json).await {
                eprintln!("Warning: HTTPS publish failed: {e}");
            }
        }

        self.last_update = Some(now);
        Ok(())
    }

    fn atomic_write_to_disk(&self, content: &str) -> Result<(), String> {
        let path = Path::new(&self.config.output_path);
        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, content)
            .map_err(|e| format!("Failed to write temporary file: {e}"))?;
        fs::rename(&tmp_path, path).map_err(|e| format!("Atomic rename failed: {e}"))?;
        Ok(())
    }
}
