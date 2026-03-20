use axum::http::HeaderMap;
use opentelemetry::propagation::Extractor;
use opentelemetry::{global, Context};
use tracing::Span;
use tracing_opentelemetry::{OpenTelemetrySpanExt, SetParentError};

// ==============================================================================
// HTTP Context Extraction
// ==============================================================================
pub struct HttpHeaderExtractor<'a>(&'a HeaderMap);

impl HttpHeaderExtractor<'_> {
    /// Extract the remote parent context from incoming HTTP headers so the request
    /// span continues the caller's trace instead of starting a disconnected tree.
    pub fn extract_remote_context(headers: &HeaderMap) -> Context {
        global::get_text_map_propagator(|propagator| {
            propagator.extract(&HttpHeaderExtractor(headers))
        })
    }

    /// Attach the propagated parent context to a freshly created tracing span.
    ///
    /// This keeps HTTP-specific carrier logic inside the telemetry crate so the
    /// gateway only decides when a remote parent should be honored.
    pub fn attach_remote_parent(span: &Span, headers: &HeaderMap) -> Result<(), SetParentError> {
        span.set_parent(Self::extract_remote_context(headers))
    }
}

impl Extractor for HttpHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}
