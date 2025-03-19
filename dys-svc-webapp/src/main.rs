use std::convert::Infallible;
use axum::{extract::Request, http::{header, HeaderValue, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, Json, Router};
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use dys_observability::logger::LoggerOptions;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use dys_nats::rpc::client::NatsRpcClient;
use dys_nats::connection::make_client;
use dys_observability::middleware::handle_shutdown_signal;

use dys_protocol::http as proto_http;
use dys_protocol::nats as proto_nats;

const DEFAULT_DIST_PATH: &str = "dys-svc-webapp/frontend/dist";

async fn static_cache_control(request: Request, next: Next) -> Response {
    // Allow wasm to be cached - too big
    if request.uri().path().ends_with(".wasm.gz") {
        return next.run(request).await;
    }

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
    let request = proto_nats::match_results::MatchRequest {
        match_ids: vec![], // ZJ-TODO: make this field useful
    };

    let mut client = proto_nats::match_results::summary_svc::MatchesRpcClient::new(
        app_state.nats_client.clone(),
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state))]
async fn query_world_state(State(app_state): State<AppState>) -> Result<Response, Infallible> {
    let request = proto_nats::world::WorldStateRequest {
        revision: 0,
    };

    let mut client = proto_nats::world::world_svc::WorldStateRpcClient::new(
        app_state.nats_client.clone(),
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state, request))]
async fn create_account(
    State(app_state): State<AppState>,
    request: Bytes
) -> Result<Response, Infallible> {
    let http_request: proto_http::auth::CreateAccountRequest = serde_json::from_slice(request.as_ref()).unwrap();
    let request = proto_nats::auth::CreateAccountRequest {
        account_name: http_request.account_name,
    };

    let mut client = proto_nats::auth::account_svc::CreateAccountRpcClient::new(
        app_state.nats_client.clone()
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_voting_proposals(
    State(app_state): State<AppState>,
    _: Bytes,
) -> Result<Response, Infallible> {
    let request = proto_nats::vote::GetProposalsRequest {};

    let mut client = proto_nats::vote::vote_svc::GetProposalsRpcClient::new(
        app_state.nats_client.clone()
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state, request))]
async fn submit_vote(
    State(app_state): State<AppState>,
    request: Bytes,
) -> Result<Response, Infallible> {
    let http_request: proto_http::vote::VoteOnProposalRequest = serde_json::from_slice(request.as_ref()).unwrap();
    let request = proto_nats::vote::VoteOnProposalRequest {
        proposal_id: http_request.proposal_id,
        option_id: http_request.option_id,
        proposal_payload: http_request.proposal_payload,
    };

    let mut client = proto_nats::vote::vote_svc::VoteOnProposalRpcClient::new(
        app_state.nats_client.clone()
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_season(
    State(app_state): State<AppState>,
    _: Bytes,
) -> Result<Response, Infallible> {
    let request = proto_nats::world::GetSeasonRequest {};

    let mut client = proto_nats::world::schedule_svc::GetSeasonRpcClient::new(
        app_state.nats_client.clone()
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
}

#[tracing::instrument(skip(app_state))]
async fn get_game_log(
    State(app_state): State<AppState>,
    Path(match_id): Path<u64>,
) -> Result<Response, Infallible> {
    let request = proto_nats::match_results::GetGameLogRequest {
        match_id
    };

    let mut client = proto_nats::match_results::summary_svc::GetGameLogRpcClient::new(
        app_state.nats_client.clone()
    );

    match client.send_request(request).await {
        Ok(resp) => Ok(Json(resp.to_http()).into_response()),
        Err(err) => Ok((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()),
    }
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

    let nats_client = make_client(Default::default()).await;

    let app_state = AppState {
        nats_client,
    };

    let app = Router::new()
        .nest_service(
            "/api",
            Router::new()
                .route("/summaries", get(query_latest_games))
                .route("/game_log/:match_id", get(get_game_log))
                .route("/world_state", get(query_world_state))
                .route("/season", get(get_season))
                .route("/create_account", post(create_account))
                .route("/get_voting_proposals", get(get_voting_proposals))
                .route("/vote", post(submit_vote))
                .with_state(app_state.clone())
        )
        .nest_service(
            "/health",
            ServiceBuilder::new()
                .service_fn(health_check)
        )
        .route_service("/", ServeFile::new(format!("{dist_path}/index.html")))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(static_cache_control))
                .service(
                    ServeDir::new(format!("{dist_path}"))
                        .not_found_service(ServeFile::new(format!("{dist_path}/index.html")))
                )
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6080").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(handle_shutdown_signal())
        .await
        .unwrap();
}