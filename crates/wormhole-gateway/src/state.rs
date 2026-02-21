use std::sync::Arc;
use typed_builder::TypedBuilder;
use wormhole_redirector::redirector::Redirector;
use wormhole_shortener::shortener::Shortener;

#[derive(Clone, TypedBuilder)]
pub struct AppState {
    /// The shortener service used to create and manage short URLs.
    #[builder(
        setter(
            fn transform<T: Shortener>(shortener: T) -> Arc<dyn Shortener> {
                 Arc::new(shortener)
            }
        )
    )]
    shortener: Arc<dyn Shortener>,
    /// The redirector service used to resolve short URLs to their original URLs.
    #[builder(
        setter(
            fn transform<T: Redirector>(redirector: T) -> Arc<dyn Redirector> {
                 Arc::new(redirector)
            }
        )
    )]
    redirector: Arc<dyn Redirector>,
    /// The base URL for public access to the short URLs.
    #[builder]
    base_url: String,
}

impl AppState {}
