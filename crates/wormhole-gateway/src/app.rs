use axum::routing::{get, post};
use axum::Router;

use crate::handlers::{create_url_handler, delete_url_handler, get_url_handler, health_handler};
use crate::state::AppState;

pub struct App {}

impl App {
    pub fn router(state: AppState) -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .nest(
                "/v1/urls",
                Router::new().route("/", post(create_url_handler)).route(
                    "/:short_code",
                    get(get_url_handler).delete(delete_url_handler),
                ),
            )
            .with_state(state)
    }
}
