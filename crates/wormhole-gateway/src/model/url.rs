use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUrlRequest {
    pub original_url: String,
    pub custom_alias: Option<String>,
    pub expire_at: Option<String>,
}

#[derive(Serialize)]
pub struct CreateUrlResponse {
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub expire_at: Option<String>,
}

#[derive(Serialize)]
pub struct DeleteUrlResponse {}

#[derive(Serialize)]
pub struct GetUrlResponse {
    pub original_url: String,
    pub expire_at: Option<String>,
}
