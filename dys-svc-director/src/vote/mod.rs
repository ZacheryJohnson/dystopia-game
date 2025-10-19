pub mod proposal;

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/proposal", api = proposal::ProposalApi),
    ),
)]
pub(crate) struct VoteApi;