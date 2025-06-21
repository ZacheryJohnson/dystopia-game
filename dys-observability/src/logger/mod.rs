use opentelemetry_otlp::{self, SpanExporter, WithExportConfig};
use opentelemetry::KeyValue;
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry_sdk::{propagation::TraceContextPropagator, runtime, trace::Config, Resource};
use opentelemetry_sdk::trace::TracerProviderBuilder;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};
use tracing_subscriber::filter::FilterExt;

pub struct LoggerOptions {
    pub application_name: String,
    pub log_level: Level,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self { 
            application_name: String::new(),
            log_level: Level::INFO,
        }
    }
}

pub fn initialize(logger_options: LoggerOptions) {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

    let env_filter = EnvFilter::from_default_env()
        .add_directive(logger_options.log_level.into());

    let format = tracing_subscriber::fmt::format().with_ansi(
        std::env::var("NO_FMT").is_err()
    );
    let format_layer = tracing_subscriber::fmt::layer().event_format(format);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(format_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let application_name = logger_options.application_name;
    let otel_endpoint = std::env::var("OTEL_ENDPOINT").unwrap_or_default();
    let tracer_provider = {
        let exporter = SpanExporter::builder()
            .with_tonic()
            .with_endpoint(otel_endpoint.clone())
            .build()
            .expect("failed to build span exporter");

        let provider = TracerProviderBuilder::default()
            .with_resource(
                Resource::builder_empty()
                    .with_attribute(
                        KeyValue::new("service.name", application_name.clone())
                    )
                    .build()
            )
            .with_batch_exporter(exporter)
            .build();

        provider
    };

    opentelemetry::global::set_tracer_provider(tracer_provider);

    tracing::info!("Using OTEL_ENDPOINT={otel_endpoint}");
}
