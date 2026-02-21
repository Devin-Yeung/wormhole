use serde::{Deserialize, Serialize};
use wormhole_core::{ShortCode, UrlRecord};

#[derive(Deserialize)]
pub struct CreateUrlRequest {
    pub original_url: String,
    pub custom_alias: Option<String>,
    pub expire_at: Option<String>,
}

#[derive(Serialize)]
pub struct UrlResponse {
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub expire_at: Option<String>,
}

impl UrlResponse {
    pub fn from_record(base_url: &str, short_code: &ShortCode, record: UrlRecord) -> Self {
        Self {
            short_code: short_code.to_string(),
            short_url: short_code.to_url(base_url),
            original_url: record.original_url,
            expire_at: record.expire_at.map(|ts| ts.to_string()),
        }
    }
}
