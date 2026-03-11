use crate::handlers::{create_url_handler, delete_url_handler, get_url_handler, health_handler};
use crate::state::AppState;
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tower_http::LatencyUnit;
use tracing::Level;

pub struct App {}

impl App {
    pub fn router(state: AppState) -> Router {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http()
            // Include request headers in the span for better observability
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            // Log incoming requests at INFO level
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            // Log outgoing responses at INFO level and include latency in microseconds
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            );

        Router::new()
            .route("/health", get(health_handler))
            .nest(
                "/v1/urls",
                Router::new().route("/", post(create_url_handler)).route(
                    "/{short_code}",
                    get(get_url_handler).delete(delete_url_handler),
                ),
            )
            .layer(trace_layer)
            .with_state(state)
    }
}
