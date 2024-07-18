use std::convert::Infallible;

use axum::{extract::Request, http::{header, HeaderValue, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, Router};
use tower::{service_fn, ServiceBuilder};
use tower_http::services::{ServeDir, ServeFile};

const DEFAULT_DIST_PATH: &'static str = "dys-svc-webapp/frontend/dist";

async fn static_cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}

async fn query_latest_games(_: Request) -> Result<Response, Infallible> {
    // ZJ-TODO: tomorrow:
    //   - conditionally make HTTP request to localhost/k8s svc here
    //   - return that data to client
    let director_api_base_uri = std::env::var("SVC_DIRECTOR_API_BASE_URI").unwrap_or(String::from("http://localhost:6081"));
    let request_url = format!("{director_api_base_uri}/latest_games");

    tracing::info!("Requesting latest games from director...");
    let Ok(response) = reqwest::get(request_url).await else {
        tracing::warn!("Failed to get latest_games from {}", director_api_base_uri);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get latest_games").into_response());
    };

    let Ok(response_body) = response.text().await else {
        tracing::warn!("Failed to get latest_games response content");
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get latest_games").into_response());
    };

    tracing::info!("Sending response...");
    let json = axum::Json(response_body);
    Ok(json.into_response())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().json().init();
    tracing::info!("Starting server...");
    let dist_path = std::env::var("DIST_PATH").unwrap_or(DEFAULT_DIST_PATH.to_string());

    // ZJ-TODO: not everything should be uncached - would be helpful to cache the game logs in particular
    let app = Router::new()
        .nest_service(
            "/api/latest_games",
            service_fn(query_latest_games)
        )
        .nest_service(
            "/assets",
            ServiceBuilder::new()
                .layer(middleware::from_fn(static_cache_control))
                .service(ServeDir::new(format!("{dist_path}/assets")))
        )
        .nest_service(
            "/",
            ServiceBuilder::new()
                .layer(middleware::from_fn(static_cache_control))
                .service(ServeDir::new(format!("{dist_path}/")))
        )
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(static_cache_control))
                .service(ServeFile::new(format!("{dist_path}/index.html")))
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}