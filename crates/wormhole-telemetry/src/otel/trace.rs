use crate::otel::resource::build_resource;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::error::Error;
// ==============================================================================
// Trace Exporter Wiring
// ==============================================================================

/// Build the tracer provider only when the process is explicitly configured to
/// export spans. This preserves the existing "JSON logs only" local workflow.
pub(crate) fn build_tracer_provider(
    service: &'static str,
) -> Result<SdkTracerProvider, Box<dyn Error>> {
    // TODO: support selecting HTTP export when environments do not expose gRPC.
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_resource(build_resource(service))
        .with_batch_exporter(exporter)
        .build();

    Ok(tracer_provider)
}
