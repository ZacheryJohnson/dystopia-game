pub mod state;

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/state", api = state::WorldStateApi),
    ),
)]
pub(crate) struct WorldApi;