use async_nats::ConnectOptions;
use tower::ServiceBuilder;
use dys_datastore::datastore::Datastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_nats::error::NatsError;
use dys_nats::router::NatsRouter;
use dys_observability::logger::LoggerOptions;
use dys_protocol::nats::auth::account_svc::{CreateAccountRpcServer, LoginRpcServer};
use dys_protocol::nats::auth::{CreateAccountRequest, CreateAccountResponse, LoginRequest, LoginResponse};

#[derive(Clone, Debug)]
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
        .service(CreateAccountRpcServer::with_handler_and_state(create_account, app_state.clone()))
        .service(LoginRpcServer::with_handler_and_state(login, app_state.clone()));
    nats.run().await;
}

#[tracing::instrument(skip(app_state))]
async fn create_account(
    request: CreateAccountRequest,
    mut app_state: AppState,
) -> Result<CreateAccountResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();

    let _: i32 = valkey.sadd(
        "env:dev:auth:accounts",
        request.account_name,
    ).await.unwrap();

    Ok(CreateAccountResponse{})
}

async fn login(
    request: LoginRequest,
    mut app_state: AppState,
) -> Result<LoginResponse, NatsError> {
    Ok(LoginResponse{})
}