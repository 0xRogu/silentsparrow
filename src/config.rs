use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub interval_hours: u64,
    #[serde(default = "default_output_path")]
    pub output_path: String,
    #[serde(default = "default_message_normal")]
    pub message_normal: String,
    #[serde(default = "default_message_overdue")]
    pub message_overdue: String, // Used by watchdog binary
    pub publish_url: Option<String>,
    pub publish_token: Option<String>,
}

fn default_interval_hours() -> u64 {
    24
}
fn default_output_path() -> String {
    "sparrow-song.json".to_string()
}
fn default_message_normal() -> String {
    "All systems chirping normally:)".to_string()
}
fn default_message_overdue() -> String {
    "The nest has gone quiet:(".to_string()
}

impl Config {
    pub fn load_or_default(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;
            let mut config: Config =
                toml::from_str(&content).map_err(|e| format!("Invalid TOML in config: {}", e))?;
            if config.interval_hours == 0 {
                config.interval_hours = default_interval_hours();
            }
            Ok(config)
        } else {
            Ok(Config {
                interval_hours: default_interval_hours(),
                output_path: default_output_path(),
                message_normal: default_message_normal(),
                message_overdue: default_message_overdue(),
                publish_url: None,
                publish_token: None,
            })
        }
    }
    pub fn interval_duration(&self) -> Duration {
        Duration::from_secs(self.interval_hours * 3600)
    }
}
