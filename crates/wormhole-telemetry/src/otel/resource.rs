use opentelemetry_sdk::Resource;
pub fn build_resource(service_name: &'static str) -> Resource {
    let mut builder = Resource::builder();

    if std::env::var_os("OTEL_SERVICE_NAME").is_none() {
        builder = builder.with_service_name(service_name);
    }

    builder.build()
}
