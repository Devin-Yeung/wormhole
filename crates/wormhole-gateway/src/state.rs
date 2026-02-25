use crate::backend::UrlService;
use std::sync::Arc;
use typed_builder::TypedBuilder;

#[derive(Clone, TypedBuilder)]
pub struct AppState {
    /// The shortener service used to create and manage short URLs.
    #[builder(
        setter(
            fn transform<T: UrlService>(t: T) -> Arc<dyn UrlService> {
                 Arc::new(t)
            }
        )
    )]
    url_service: Arc<dyn UrlService>,
    /// The base URL for public access to the short URLs.
    #[builder]
    base_url: String,
}

impl AppState {
    pub fn url_service(&self) -> Arc<dyn UrlService> {
        Arc::clone(&self.url_service)
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
