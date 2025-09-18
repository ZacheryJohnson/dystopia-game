use utoipa_swagger_ui::SwaggerUi;
use dys_observability::middleware::handle_shutdown_signal;

#[tokio::main]
async fn main() {
    let api_spec_str = std::fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/generated/openapi.json")
    ).unwrap();

    let api_spec: utoipa::openapi::OpenApi = serde_json::from_str(&api_spec_str).unwrap();

    let (mut router, api) = utoipa_axum::router::OpenApiRouter::with_openapi(api_spec).split_for_parts();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:6050").await.unwrap();

    router = router.merge(
        SwaggerUi::new("/swagger")
            .url("/api/openapi.json", api)
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}
