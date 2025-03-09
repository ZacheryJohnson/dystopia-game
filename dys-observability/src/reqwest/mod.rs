use opentelemetry_http::HeaderInjector;
use reqwest::{Error, Method, Response};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;


/// Sends a GET request to the provided URL with headers that enable tracing.
pub async fn get(request_url: impl Into<String>) -> Result<Response, Error> {
    let http_client = reqwest::Client::builder()
        .build()?;

    let mut request = http_client
        .request(Method::GET, request_url.into())
        .build()?;
    
    opentelemetry::global::get_text_map_propagator(|propogator| {
        propogator.inject_context(&Span::current().context(), &mut HeaderInjector(request.headers_mut()));
    });
    
    http_client.execute(request).await
}

pub async fn post(request_url: impl Into<String>, body: String) -> Result<Response, Error> {
    let http_client = reqwest::Client::builder()
        .build()?;

    let mut request = http_client
        .request(Method::POST, request_url.into())
        .body(body)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .build()?;

    opentelemetry::global::get_text_map_propagator(|propogator| {
        propogator.inject_context(&Span::current().context(), &mut HeaderInjector(request.headers_mut()));
    });

    http_client.execute(request).await
}