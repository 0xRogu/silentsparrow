mod canary;
mod crypto;
mod publisher;

use canary::Canary;
use crypto::Crypto;
use publisher::Publisher;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let crypto = Crypto::new();
    let publisher = Publisher::new("https://example.com/status/sparrow");

    let message = "All systems chirping normally.";
    let signature_bytes = crypto.sign(message);
    let canary = Canary::new(message, &signature_bytes, None);

    match canary.to_json() {
        Ok(json) => {
            println!("Generated Canary:\n{}", json);
            println!("Public Key: {}", crypto.public_key_hex());

            if let Err(e) = publisher.publish(&json).await {
                eprintln!("Failed to publish canary: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to serialize canary: {}", e),
    }
    loop {
        sleep(Duration::from_secs(10)).await;
        let new_signature = crypto.sign(message);
        let new_canary = Canary::new(message, &new_signature, None);
        match new_canary.to_json() {
            Ok(json) => {
                println!("Updated Canary: \n{}", json);
                if let Err(e) = publisher.publish(&json).await {
                    eprintln!("Failed to publish canary: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to serialize canary: {}", e),
        }
    }
}
