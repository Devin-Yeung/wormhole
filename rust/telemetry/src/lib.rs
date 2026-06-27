mod extractor;
mod otel;

use opentelemetry::global;
use std::error::Error;
use tracing_log::LogTracer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub use crate::extractor::grpc::GrpcMetadataExtractor;
pub use crate::extractor::http::HttpHeaderExtractor;
pub use crate::otel::TelemetryGuard;
use opentelemetry_sdk::propagation::TraceContextPropagator;

// ==============================================================================
// Subscriber Initialization
// ==============================================================================
///
/// Initialize structured logging and optional OTLP span exporting for a
/// service binary.
///
/// We always keep the structured JSON log layer because it is already part of
/// the service's operational surface. OTLP exporting is added only when the
/// standard `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is configured.
pub fn init_tracing(service_name: &'static str) -> Result<TelemetryGuard, Box<dyn Error>> {
    LogTracer::init()?;

    // W3C TraceContext is the common denominator between the HTTP edge and the
    // downstream gRPC calls that the gateway makes on behalf of the request.
    global::set_text_map_propagator(TraceContextPropagator::new());

    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
    let fmt_layer = tracing_subscriber::fmt::layer().json();

    let telemetry = TelemetryGuard::new(service_name)?;

    let subscriber = Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .with(telemetry.otel_trace_layer());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(telemetry)
}
