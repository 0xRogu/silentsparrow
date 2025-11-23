use std::error::Error;

pub struct Publisher {
    endpoint: String,
}

impl Publisher {
    pub fn new(endpoint: &str) -> Self {
        Publisher {
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn publish(&self, canary_json: &str) -> Result<(), Box<dyn Error>> {
        println!("Publishing canary to {}:{}", self.endpoint, canary_json);

        Ok(())
    }
}
