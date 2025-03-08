use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use dys_datastore::datastore::Datastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_observability::logger::LoggerOptions;
use dys_observability::middleware::{handle_shutdown_signal, make_span, map_trace_context, record_trace_id};
use dys_protocol::protocol::auth::{CreateAccountRequest, LoginRequest};

#[derive(Clone)]
struct AppState {
    valkey: ValkeyDatastore,
}

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "auth".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");

    let valkey_config = ValkeyConfig::new(
        std::env::var("VALKEY_USER").unwrap_or(String::from("default")),
        std::env::var("VALKEY_PASS").unwrap_or(String::from("")),
        std::env::var("VALKEY_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("VALKEY_PORT").unwrap_or(String::from("6379")).parse::<u16>().unwrap()
    );

    let app_state = AppState {
        valkey: *ValkeyDatastore::connect(valkey_config).await.unwrap(),
    };

    let trace_middleware_layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_grpc().make_span_with(make_span))
        .map_request(map_trace_context)
        .map_request(record_trace_id);

    let app = Router::new()
        .route("/create_account", post(create_account))
        .route("/login", post(login))
        .route("/health", get(health_check))
        .layer(trace_middleware_layer)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6082").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}

async fn health_check(
    State(_): State<AppState>
) -> Result<impl IntoResponse, StatusCode> {
    Ok(StatusCode::OK)
}

async fn create_account(
    State(mut app_state): State<AppState>,
    request: String,
) -> impl IntoResponse {
    tracing::info!("Creating account!");
    let mut valkey = app_state.valkey.connection();

    let request: CreateAccountRequest = serde_json::from_str(&request).unwrap();

    let resp: i32 = valkey.sadd(
        "env:dev:auth:accounts",
        request.account_name,
    ).await.unwrap();

    if resp == 0 {
        StatusCode::BAD_REQUEST
    } else {
        StatusCode::OK
    }
}

async fn login(
    State(mut app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
    StatusCode::NOT_IMPLEMENTED
}