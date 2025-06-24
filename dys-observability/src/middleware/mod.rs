use axum::{body::Body, extract::Request};
use opentelemetry::trace::TraceContextExt;
use opentelemetry_http::HeaderExtractor;
use tokio::signal;
use tracing::{info_span, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn map_trace_context(request: Request<Body>) -> Request<Body> {
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });

    tracing::Span::current().set_parent(parent_context);

    request
}

pub fn record_trace_id(request: Request<Body>) -> Request<Body> {
    let tokio_span_context = Span::current().context();
    let span = tokio_span_context.span();
    let trace_id = span.span_context().trace_id();
    Span::current().record("trace_id", trace_id.to_string());

    request
}

pub fn make_span(request: &Request<Body>) -> Span {
    tracing::info!("temp: making span!");

    let headers = request.headers();
    info_span!("incoming http request", ?headers, trace_id = tracing::field::Empty)
}

pub async fn handle_shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::warn!("received ctrl+c...") },
        _ = terminate => { tracing::warn!("received terminate...") },
    }
}