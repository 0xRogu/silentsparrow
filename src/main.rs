mod canary;
mod crypto;
mod publisher;
mod config;

use canary::Canary;
use config::Config;
use std::path::PathBuf;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = std::env::args()
        .find(|arg| arg.starts_with("--config="))
        .and_then(|arg| arg.strip_prefix("--config=").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("sparrow.toml"));
    let config = Config::load_or_default(&config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;
        println!("Silent Sparrow started - updating every {} hour(s)", config.interval_hours);
        println!("Output file: {}", config.output_path);
        if config.publish_url.is_some() {
            println!("HTTPS publishing enabled");
        }
    let mut canary = Canary::new(config.clone());
    if let Err(e) = canary.refresh().await {
        eprintln!("Initial refresh failed: {}", e);
    } else {
        println!("Initial sparrow song written");
    }
    let mut interval = tokio::time::interval(config.interval_duration());
    loop {
        interval.tick().await;
        if let Err(e) = canary.refresh().await {
            eprintln!("Scheduled refresh failed: {}", e);
        } else {
            println!("Scheduled update completed at {}", chrono::Utc::now().to_rfc3339());
        }
    }
}
