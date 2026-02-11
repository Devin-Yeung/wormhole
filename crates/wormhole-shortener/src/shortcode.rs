use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortCode(String);

impl ShortCode {
    pub fn new(code: String) -> Self {
        Self(code)
    }

    /// Generates the full shortened URL based on the provided base URL.
    pub fn url(&self, base_url: &str) -> String {
        return format!("{}/{}", base_url.trim_end_matches('/'), self.0);
    }
}
