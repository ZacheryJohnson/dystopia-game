use opentelemetry_otlp::{self, WithExportConfig};
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::Config, Resource};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn initialize(application_name: impl Into<String>) {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let otel_endpoint = std::env::var("OTEL_ENDPOINT").unwrap_or_default();

    let application_name = application_name.into();

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(otel_endpoint.clone())
        )
        .with_trace_config(Config::default()
            .with_resource(Resource::new(vec![
                KeyValue::new("service.name", application_name.clone())
            ]))
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Couldn't create OTLP tracer")
        .tracer(application_name);

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    let format = tracing_subscriber::fmt::format();
    let format_layer = tracing_subscriber::fmt::layer().event_format(format);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(format_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    tracing::info!("Using OTEL_ENDPOINT={otel_endpoint}");
}