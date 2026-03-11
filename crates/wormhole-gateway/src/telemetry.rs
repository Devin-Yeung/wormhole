use axum::http::HeaderMap;
use opentelemetry::global;
use opentelemetry::propagation::Extractor;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::{Context, KeyValue};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::env;
use std::error::Error;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const SERVICE_NAME: &str = "wormhole-gateway";

// ==============================================================================
// Subscriber Initialization
// ==============================================================================

/// Owns the OTEL tracer provider so the binary can flush spans before exit.
pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
}

impl TelemetryGuard {
    pub fn shutdown(&mut self) {
        if let Some(tracer_provider) = self.tracer_provider.take() {
            if let Err(error) = tracer_provider.shutdown() {
                eprintln!("failed to shut down OpenTelemetry tracer provider: {error}");
            }
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Initialize tracing for the gateway binary.
///
/// We always keep the structured JSON log layer because it is already part of
/// the gateway's operational surface. OTLP exporting is added only when the
/// standard `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is configured.
pub fn init_tracing() -> Result<TelemetryGuard, Box<dyn std::error::Error>> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    // W3C TraceContext is the common denominator between the HTTP edge and the
    // downstream gRPC calls that the gateway makes on behalf of the request.
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer_provider = build_tracer_provider()?;
    let otel_layer = tracer_provider
        .as_ref()
        .map(|provider| tracing_opentelemetry::layer().with_tracer(provider.tracer(SERVICE_NAME)));

    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(otel_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    Ok(TelemetryGuard { tracer_provider })
}

fn build_tracer_provider() -> Result<Option<SdkTracerProvider>, Box<dyn Error>> {
    if env::var_os("OTEL_EXPORTER_OTLP_ENDPOINT").is_none() {
        return Ok(None);
    }

    // Start from the SDK's resource detectors so operators can still enrich the
    // service with `OTEL_RESOURCE_ATTRIBUTES` without code changes.
    let mut resource_builder = Resource::builder()
        .with_attributes([KeyValue::new("service.version", env!("CARGO_PKG_VERSION"))]);

    if env::var_os("OTEL_SERVICE_NAME").is_none() {
        resource_builder = resource_builder.with_service_name(SERVICE_NAME);
    }

    // TODO: we might not always use grpc endpoint
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(resource_builder.build())
        .with_batch_exporter(exporter)
        .build();

    Ok(Some(tracer_provider))
}

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
}

impl Extractor for HttpHeaderExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|name| name.as_str()).collect()
    }
}
