use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_service_base_macros::api;
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Vote",
        description = "ZJ-TODO: deprecate, should be part of proposal API.",
    ),
    paths(
        submit_vote,
    ),
)]
pub struct SubmitApi;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitVoteRequest {
    proposal_id: u64,
    proposal_option_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubmitVoteResponse {

}

#[api(
    request = SubmitVoteRequest,
    app_state = AppState,
    http(
        method = "Post",
        path = "",
    ),
    nats(
        topic = "api.v1.vote.submit.post",
    ),
)]
pub async fn submit_vote(
    request: SubmitVoteRequest,
    app_state: AppState,
) -> Result<SubmitVoteResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let _: i32 = valkey.hincr(
        format!("env:dev:votes:proposal:{}", request.proposal_id),
        format!("option:{}", request.proposal_option_id),
        1,
    ).await.unwrap();

    Ok(SubmitVoteResponse {})
}