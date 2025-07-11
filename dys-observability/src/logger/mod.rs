use opentelemetry_otlp::{self, LogExporter, SpanExporter, WithExportConfig};
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::{propagation::TraceContextPropagator, Resource};
use opentelemetry_sdk::logs::{LoggerProviderBuilder, SdkLoggerProvider};
use opentelemetry_sdk::trace::{SdkTracerProvider, TracerProviderBuilder};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use tracing_subscriber::util::SubscriberInitExt;

pub struct LoggerOptions {
    pub application_name: String,
    pub log_level: Level,
    pub with_ansi: bool
}

impl Default for LoggerOptions {
    fn default() -> Self {
        Self { 
            application_name: String::new(),
            log_level: Level::INFO,
            with_ansi: true,
        }
    }
}

fn get_otel_logging_provider(
    endpoint: impl Into<String>,
    application_name: impl Into<String>,
) -> SdkLoggerProvider {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.into())
        .build()
        .expect("failed to build log exporter");

    LoggerProviderBuilder::default()
        .with_resource(
            Resource::builder_empty()
                .with_attribute(
                    KeyValue::new("service.name", application_name.into())
                )
                .build()
        )
        .with_batch_exporter(exporter)
        .build()
}

fn get_otel_tracing_provider(
    endpoint: impl Into<String>,
    application_name: impl Into<String>,
) -> SdkTracerProvider {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.into())
        .build()
        .expect("failed to build span exporter");

    TracerProviderBuilder::default()
        .with_resource(
            Resource::builder_empty()
                .with_attribute(
                    KeyValue::new("service.name", application_name.into())
                )
                .build()
        )
        .with_batch_exporter(exporter)
        .build()
}

pub fn initialize(logger_options: LoggerOptions) {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::default());

    let application_name = logger_options.application_name;
    let otel_endpoint = std::env::var("OTEL_ENDPOINT").unwrap_or_default();

    let env_filter = EnvFilter::from_default_env()
        .add_directive(logger_options.log_level.into());

    let format = tracing_subscriber::fmt::format().with_ansi(logger_options.with_ansi);
    let format_layer = tracing_subscriber::fmt::layer()
        .with_ansi(logger_options.with_ansi)
        .event_format(format);

    if otel_endpoint.is_empty() {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(format_layer)
            .init();
    } else {
        let tracing_provider = get_otel_tracing_provider(
            &otel_endpoint,
            &application_name,
        );
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(tracing_provider.tracer(application_name.clone()));

        let logging_layer = OpenTelemetryTracingBridge::new(&get_otel_logging_provider(
            otel_endpoint.clone(),
            application_name.clone()
        ));

        tracing_subscriber::registry()
            .with(env_filter)
            .with(format_layer)
            .with(logging_layer)
            .with(telemetry_layer)
            .init();

        opentelemetry::global::set_tracer_provider(tracing_provider);
    }

    tracing::info!("Logger initialized; OTEL_ENDPOINT={otel_endpoint} log_level={}", logger_options.log_level);
}
