use axum::{http::StatusCode, routing::get_service, Router};
use tower_http::services::{ServeDir, ServeFile};

const PROD_DIST_PATH: &'static str = "dys-svc-webapp/frontend/dist";

#[tokio::main]
async fn main() {
    let app = Router::new()
        .nest_service(
            "/assets",
            get_service(ServeDir::new(format!("{PROD_DIST_PATH}/assets")))
        )
        .nest_service(
            "/",
            get_service(ServeDir::new(format!("{PROD_DIST_PATH}/")))
        )
        .fallback(
            get_service(ServeFile::new(format!("{PROD_DIST_PATH}/index.html"))).handle_error(|_| async move { (StatusCode::INTERNAL_SERVER_ERROR, "internal server error") })
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:6080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}