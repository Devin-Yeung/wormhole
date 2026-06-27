use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::{SdkTracer, SdkTracerProvider};
use std::error::Error;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

mod resource;
mod trace;
pub(crate) use trace::build_tracer_provider;

/// Owns the OTEL tracer provider so the binary can flush spans before exit.
pub struct TelemetryGuard {
    service_name: &'static str,
    tracer_provider: SdkTracerProvider,
}

impl TelemetryGuard {
    pub(crate) fn new(service_name: &'static str) -> Result<Self, Box<dyn Error>> {
        let tracer_provider = build_tracer_provider(service_name)?;

        Ok(Self {
            service_name,
            tracer_provider,
        })
    }

    /// Build the optional OTEL layer from the owned provider so subscriber
    /// setup does not need to know how tracers are derived from the SDK.
    pub(crate) fn otel_trace_layer<S>(&self) -> OpenTelemetryLayer<S, SdkTracer>
    where
        S: Subscriber,
        for<'span> S: LookupSpan<'span>,
    {
        tracing_opentelemetry::layer().with_tracer(self.tracer_provider.tracer(self.service_name))
    }

    pub fn shutdown(&mut self) {
        // Flush any buffered spans before the process exits.
        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("failed to shutdown telemetry tracer provider: {e}");
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        self.shutdown();
    }
}
