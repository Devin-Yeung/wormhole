use std::sync::Arc;

use wormhole_shortener::shortener::Shortener;

#[derive(Clone)]
pub struct AppState {
    shortener: Arc<dyn Shortener>,
    base_url: String,
}

impl AppState {
    pub fn new(shortener: Arc<dyn Shortener>, public_base_url: impl Into<String>) -> Self {
        Self {
            shortener,
            base_url: public_base_url.into(),
        }
    }
}
