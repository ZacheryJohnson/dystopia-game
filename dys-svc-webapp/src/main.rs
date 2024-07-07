use axum::{extract::Request, http::{header, HeaderValue}, middleware::{self, Next}, response::Response, Router};
use tower::ServiceBuilder;
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

#[tokio::main]
async fn main() {
    println!("Starting server...");
    let dist_path = std::env::var("DIST_PATH").unwrap_or(DEFAULT_DIST_PATH.to_string());

    // ZJ-TODO: not everything should be uncached - would be helpful to cache the game logs in particular
    let app = Router::new()
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