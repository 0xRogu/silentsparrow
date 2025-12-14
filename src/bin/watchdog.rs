use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use silent_sparrow::Config;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::time::{Duration, sleep};

#[derive(Serialize, Deserialize)]
struct SparrowSong {
    timestamp: String,
    message: String,
    signature: String,
    public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    log_hash: Option<String>,
}

#[tokio::main]
async fn main() {
    let config_path = std::env::args()
        .find(|arg| arg.starts_with("--config="))
        .and_then(|arg| arg.strip_prefix("--config=").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("sparrow.toml"));

    let config = Config::load_or_default(&config_path).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to load config ({}), using defaults", e);
        Config {
            interval_hours: 24,
            output_path: "sparrow-song.json".to_string(),
            message_normal: "All systems chirping normally:)".to_string(),
            message_overdue: "The nest has gone quiet:(".to_string(),
            publish_url: None,
            publish_token: None,
        }
    });

    let canary_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "sparrow-song.json".to_string());

    let max_age_hours: i64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or((config.interval_hours + 1) as i64); // Default: interval + 1 hour

    let check_interval_secs: u64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(300); // Check every 5 minutes

    println!(
        "Watchdog started - monitoring {} every {} seconds",
        canary_path, check_interval_secs
    );
    println!("Will mark as overdue if older than {} hours", max_age_hours);
    println!("Normal message: {}", config.message_normal);
    println!("Overdue message: {}", config.message_overdue);

    loop {
        if let Err(e) = check_and_update(&canary_path, max_age_hours, &config).await {
            eprintln!("Watchdog error: {}", e);
        }
        sleep(Duration::from_secs(check_interval_secs)).await;
    }
}

async fn check_and_update(path: &str, max_age_hours: i64, config: &Config) -> Result<(), String> {
    // Read current canary
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read canary file: {}", e))?;

    let mut song: SparrowSong = serde_json::from_str(&content)
        .map_err(|e| format!("Invalid JSON in canary file: {}", e))?;

    let timestamp = DateTime::parse_from_rfc3339(&song.timestamp)
        .map_err(|e| format!("Invalid timestamp: {}", e))?
        .with_timezone(&Utc);

    let now = Utc::now();
    let age = now.signed_duration_since(timestamp);
    let max_age = chrono::Duration::hours(max_age_hours);

    // Check if canary is stale
    let is_stale = age > max_age;

    // Determine what the message should be
    let expected_message = if is_stale {
        &config.message_overdue
    } else {
        &config.message_normal
    };

    // Only update if the message is wrong
    if song.message != *expected_message {
        song.message = expected_message.to_string();

        // Note: Signature is now invalid, but that's OK - this is an emergency update
        // The signature was from the original process, and we're overriding the message
        song.signature = String::from("WATCHDOG_OVERRIDE");

        let json = serde_json::to_string_pretty(&song)
            .map_err(|e| format!("JSON serialization failed: {}", e))?;

        atomic_write(path, &json)?;

        if is_stale {
            eprintln!(
                "⚠️  ALERT: Canary is {} old - updated message to overdue",
                format_duration(age)
            );
        } else {
            println!(
                "✓ Canary recovered - updated message to normal (age: {})",
                format_duration(age)
            );
        }
    }

    Ok(())
}

fn atomic_write(path: &str, content: &str) -> Result<(), String> {
    let path = Path::new(path);
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content).map_err(|e| format!("Failed to write temporary file: {}", e))?;
    fs::rename(&tmp_path, path).map_err(|e| format!("Atomic rename failed: {}", e))?;
    Ok(())
}

fn format_duration(d: chrono::Duration) -> String {
    let hours = d.num_hours();
    let minutes = d.num_minutes() % 60;
    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}
