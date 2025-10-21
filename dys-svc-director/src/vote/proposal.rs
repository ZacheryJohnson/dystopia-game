use utoipa::openapi::path::{Parameter, ParameterIn};
use utoipa::{IntoParams, ToSchema};
use serde::{Deserialize, Serialize};
use dys_datastore_valkey::datastore::AsyncCommands;
use dys_nats::error::NatsError;
use dys_service_base_macros::{api, ApiRequest};
use dys_world::proposal::Proposal;
use crate::AppState;

#[derive(utoipa::OpenApi)]
#[openapi(
    info(
        title = "Proposals",
        description = "World state as of the current simulation",
    ),
    paths(
        get_voting_proposals,
    ),
)]
pub struct ProposalApi;

#[derive(Debug, Clone, Serialize, Deserialize, ApiRequest)]
pub struct GetProposalsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GetProposalsResponse {
    pub proposals: Vec<Proposal>,
}

#[api(
    request = GetProposalsRequest,
    response = GetProposalsResponse,
    error = NatsError,
    app_state = AppState,
    http(
        method = "Get",
        path = "",
    ),
    nats(
        topic = "api.v1.vote.proposal.get",
    ),
)]
pub async fn get_voting_proposals(
    _: GetProposalsRequest,
    app_state: AppState,
) -> Result<GetProposalsResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let proposal_jsons: Vec<String> = valkey.hvals(
        "env:dev:proposals:latest"
    ).await.unwrap();

    let proposals = proposal_jsons
        .iter()
        .map(|proposal_str| serde_json::from_str(proposal_str).unwrap())
        .collect::<Vec<Proposal>>();

    let response = GetProposalsResponse {
        proposals
    };

    Ok(response)
}
