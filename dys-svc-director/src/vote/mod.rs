pub mod proposal;
pub mod submit;

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/proposal", api = proposal::ProposalApi),
        (path = "/submit", api = submit::SubmitApi)
    ),
)]
pub(crate) struct VoteApi;