use crate::port::UrlApiPort;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct AppState {
    /// The shortener service used to create and manage short URLs.
    #[builder(
        setter(
            fn transform<T: UrlApiPort>(port: T) -> Arc<dyn UrlApiPort> {
                 Arc::new(port)
            }
        )
    )]
    api: Arc<dyn UrlApiPort>,
    /// The base URL for public access to the short URLs.
    #[builder]
    base_url: String,
}

impl AppState {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
