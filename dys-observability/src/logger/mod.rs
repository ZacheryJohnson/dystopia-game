use opentelemetry_otlp::{self, WithExportConfig};
use opentelemetry::{trace::TracerProvider, KeyValue};
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace::Config, Resource};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer, Registry};

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

    let application_name = logger_options.application_name;
    let otel_endpoint = std::env::var("OTEL_ENDPOINT").unwrap_or_default();
    let telemetry_layer = if !otel_endpoint.is_empty() {
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
        tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .boxed()
    } else {
        tracing_opentelemetry::layer().boxed()
    };

    let format = tracing_subscriber::fmt::format();
    let format_layer = tracing_subscriber::fmt::layer().event_format(format);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(format_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    tracing::info!("Using OTEL_ENDPOINT={otel_endpoint}");
}
