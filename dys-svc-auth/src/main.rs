use async_nats::ConnectOptions;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::routing::{get, post};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use dys_datastore::datastore::Datastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_nats::error::NatsError;
use dys_nats::router::NatsRouter;
use dys_observability::logger::LoggerOptions;
use dys_observability::middleware::{handle_shutdown_signal, make_span, map_trace_context, record_trace_id};
use dys_protocol::nats::auth::account_svc::{CreateAccountRpcService, LoginRpcService};
use dys_protocol::nats::auth::{CreateAccountRequest, CreateAccountResponse, LoginRequest, LoginResponse};

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

    let nats = NatsRouter::new()
        .await
        .service(CreateAccountRpcService::with_handler(create_account))
        .service(LoginRpcService::with_handler(login));
    nats.run().await;
}

async fn health_check(
    State(_): State<AppState>
) -> Result<impl IntoResponse, StatusCode> {
    Ok(StatusCode::OK)
}

async fn create_account(
    request: CreateAccountRequest,
) -> Result<CreateAccountResponse, NatsError> {
    tracing::info!("Creating account!");
    // let mut valkey = app_state.valkey.connection();
    //
    // let _: i32 = valkey.sadd(
    //     "env:dev:auth:accounts",
    //     request.account_name,
    // ).await.unwrap();

    Ok(CreateAccountResponse{})
}

async fn login(
    request: LoginRequest,
) -> Result<LoginResponse, NatsError> {
    tracing::info!("Logging in!");

    Ok(LoginResponse{})
}