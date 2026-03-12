use opentelemetry::propagation::Extractor;
use opentelemetry::{global, Context};
use tonic::metadata::MetadataMap;
use tracing::Span;
use tracing_opentelemetry::{OpenTelemetrySpanExt, SetParentError};

// ==============================================================================
// gRPC Context Extraction
// ==============================================================================

pub struct GrpcMetadataExtractor<'a>(&'a MetadataMap);

impl GrpcMetadataExtractor<'_> {
    /// Extract the caller's distributed tracing context from gRPC metadata.
    ///
    /// gRPC uses HTTP/2 headers under the hood, so the standard W3C
    /// propagator can read metadata values the same way it reads HTTP headers.
    pub fn extract_remote_context(metadata: &MetadataMap) -> Context {
        global::get_text_map_propagator(|propagator| {
            propagator.extract(&GrpcMetadataExtractor(metadata))
        })
    }

    /// Attach the propagated parent context to a newly-created server span.
    pub fn attach_remote_parent(span: &Span, metadata: &MetadataMap) -> Result<(), SetParentError> {
        span.set_parent(Self::extract_remote_context(metadata))
    }
}

impl Extractor for GrpcMetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.as_ref().keys().map(|name| name.as_str()).collect()
    }
}
