use crate::handlers::{create_url_handler, delete_url_handler, get_url_handler, health_handler};
use crate::state::AppState;
use crate::telemetry::HttpHeaderExtractor;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{get, post};
use axum::Router;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse};
use tower_http::LatencyUnit;
use tracing::Level;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub struct App {}

impl App {
    pub fn router(state: AppState) -> Router {
        let trace_layer = tower_http::trace::TraceLayer::new_for_http()
            // Continue upstream traces when callers provide a `traceparent`
            // header, while still capturing the edge-facing request metadata.
            .make_span_with(|request: &Request<_>| {
                let matched_path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str)
                    .unwrap_or(request.uri().path());

                let span = tracing::info_span!(
                    "http.request",
                    method = %request.method(),
                    matched_path,
                    uri = %request.uri(),
                    version = ?request.version(),
                    headers = ?request.headers(),
                );

                if let Err(error) = span.set_parent(HttpHeaderExtractor::extract_remote_context(
                    request.headers(),
                )) {
                    tracing::warn!(
                        error = %error,
                        "failed to attach remote OpenTelemetry parent to request span"
                    );
                }

                span
            })
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
