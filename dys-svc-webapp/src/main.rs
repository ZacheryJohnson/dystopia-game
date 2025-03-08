use std::convert::Infallible;
use std::time::Duration;
use axum::{extract::Request, http::{header, HeaderValue, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, Router};
use axum::extract::State;
use axum::http::Method;
use axum::routing::get;
use tonic::codegen::tokio_stream::StreamExt;
use dys_observability::{logger::LoggerOptions, middleware::{make_span, map_trace_context, record_trace_id}};
use tower::ServiceBuilder;
use tower_http::{services::{ServeDir, ServeFile}, trace::TraceLayer};
use dys_observability::middleware::handle_shutdown_signal;
use dys_protocol::protocol::match_results::{MatchRequest, MatchResponse};

const DEFAULT_DIST_PATH: &str = "dys-svc-webapp/frontend/dist";

async fn static_cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}

#[derive(Clone, Debug)]
struct AppState {
    nats_client: async_nats::Client,
}

#[tracing::instrument(skip(app_state))]
async fn query_latest_games(State(app_state): State<AppState>) -> Result<Response, StatusCode> {
    let match_request = MatchRequest {
        match_ids: vec![], // ZJ-TODO: make this field useful
    };

    let payload = postcard::to_allocvec(&match_request).unwrap();

    let reply_subject = "rpc.testing";
    let result = app_state.nats_client.subscribe(
        reply_subject
    ).await;

    let Ok(mut reply_subscriber) = result else {
        tracing::error!("failed to subscribe to reply topic {reply_subject}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let result = app_state.nats_client.publish_with_reply(
        format!("rpc.{}", dys_protocol::protocol::match_results::summary_server::SERVICE_NAME),
        reply_subject,
        payload.into()
    ).await;

    if result.is_err() {
        tracing::error!("failed to publish summary request");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

   let Ok(response) = tokio::time::timeout(Duration::from_millis(500), async {
        loop {
            let Some(response) = reply_subscriber.next().await else {
                continue;
            };

            return response;
        }
    }).await else {
        tracing::error!("timed out waiting for reply");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let response: MatchResponse = postcard::from_bytes(&response.payload).unwrap();
    let response_json = serde_json::to_string(&response).unwrap();

    let json = axum::Json(response_json);
    Ok(json.into_response())
}

#[tracing::instrument]
async fn query_combatants(_: Request) -> Result<Response, Infallible> {
    let director_api_base_uri = std::env::var("SVC_DIRECTOR_API_BASE_URI").unwrap_or(String::from("http://localhost:6081"));
    let request_url = format!("{director_api_base_uri}/combatants");

    tracing::info!("Requesting combatants from director...");
    let maybe_response = dys_observability::reqwest::get(request_url).await;
    if let Err(err) = maybe_response {
        tracing::warn!("Failed to get combatants from {}: {err:?}", director_api_base_uri);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get combatants").into_response());
    };

    let response = maybe_response.unwrap();

    let Ok(response_body) = response.text().await else {
        tracing::warn!("Failed to get combatants response content");
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get combatants").into_response());
    };

    tracing::info!("Sending response...");
    let json = axum::Json(response_body);
    Ok(json.into_response())
}

#[tracing::instrument]
async fn query_world_state(_: Request) -> Result<Response, Infallible> {
    let director_api_base_uri = std::env::var("SVC_DIRECTOR_API_BASE_URI").unwrap_or(String::from("http://localhost:6081"));
    let request_url = format!("{director_api_base_uri}/world_state");

    tracing::info!("Requesting world_state from director...");
    let maybe_response = dys_observability::reqwest::get(request_url).await;
    if let Err(err) = maybe_response {
        tracing::warn!("Failed to get world_state from {}: {err:?}", director_api_base_uri);
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get world_state").into_response());
    };

    let response = maybe_response.unwrap();

    let Ok(response_body) = response.text().await else {
        tracing::warn!("Failed to get world_state response content");
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get world_state").into_response());
    };

    tracing::info!("Sending response...");
    let json = axum::Json(response_body);
    Ok(json.into_response())
}

async fn create_account(request: Request) -> Result<Response, Infallible> {
    if request.method() != Method::POST {
        return Ok((StatusCode::METHOD_NOT_ALLOWED, "method not allowed").into_response());
    };

    let auth_api_base_uri = std::env::var("SVC_AUTH_API_BASE_URI").unwrap_or(String::from("http://localhost:6082"));
    let request_url = format!("{auth_api_base_uri}/create_account");

    let body = request.into_body();
    let bytes = axum::body::to_bytes(body, 256usize).await.unwrap();
    let body_str = String::from_utf8(bytes.to_vec()).unwrap().replace("accountName", "account_name");

    let maybe_response = dys_observability::reqwest::post(request_url, body_str).await;
    if let Err(err) = maybe_response {
        tracing::warn!("Failed to create account: {err:?}");
        return Ok((StatusCode::BAD_REQUEST, "failed to get create account").into_response());
    };

    let Ok(response_body) = maybe_response.unwrap().text().await else {
        tracing::warn!("Failed to get create account response content");
        return Ok((StatusCode::INTERNAL_SERVER_ERROR, "failed to get create account response").into_response());
    };

    tracing::info!("Sending response...");
    let json = axum::Json(response_body);
    Ok(json.into_response())
}

async fn health_check(_: Request) -> Result<impl IntoResponse, Infallible> {
    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "webapp".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");
    let dist_path = std::env::var("DIST_PATH").unwrap_or(DEFAULT_DIST_PATH.to_string());

    let nats_server_str = format!(
        "{}:{}",
        std::env::var("NATS_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("NATS_PORT").unwrap_or(String::from("4222")).parse::<u16>().unwrap(),
    );

    let nats_client = async_nats::ConnectOptions::new()
        .token(std::env::var("NATS_TOKEN").unwrap_or(String::from("replaceme")))
        .connect(nats_server_str)
        .await
        .expect("failed to connect to NATS server");

    let app_state = AppState {
        nats_client,
    };

    let app = Router::new()
        .nest_service(
            "/api/summaries",
            Router::new()
                .fallback(get(query_latest_games))
                .with_state(app_state)
        )
        .nest_service(
            "/api/combatants",
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_grpc().make_span_with(make_span))
                .map_request(map_trace_context)
                .map_request(record_trace_id)
                .layer(middleware::from_fn(static_cache_control))
                .service_fn(query_combatants)
        )
        .nest_service(
            "/api/world_state",
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_grpc().make_span_with(make_span))
                .map_request(map_trace_context)
                .map_request(record_trace_id)
                .layer(middleware::from_fn(static_cache_control))
                .service_fn(query_world_state)
        )
        .nest_service(
            "/api/create_account",
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_grpc().make_span_with(make_span))
                .map_request(map_trace_context)
                .map_request(record_trace_id)
                .service_fn(create_account)
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
        .nest_service(
            "/health",
            ServiceBuilder::new()
                .service_fn(health_check)
        )
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(static_cache_control))
                .service(ServeFile::new(format!("{dist_path}/index.html")))
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6080").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}