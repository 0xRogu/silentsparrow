use reqwest::{Client, StatusCode};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct HttpsPublisher {
    client: Client,
    url: String,
    auth_token: Option<String>,
}

impl HttpsPublisher {
    pub fn new(url: impl Into<String>, auth_token: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client");
        Self {
            client,
            url: url.into(),
            auth_token,
        }
    }

    pub async fn publish(&self, json_payload: &str) -> Result<(), String> {
        let mut request = self.client.post(&self.url).body(json_payload.to_owned());
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        }
        let response = request
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        match response.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::NO_CONTENT => Ok(()),
            status => Err(format!(
                "Server returned {}: {}",
                status,
                response.text().await.unwrap_or_default()
            )),
        }
    }
}
