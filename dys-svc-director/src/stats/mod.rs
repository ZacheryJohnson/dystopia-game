pub mod recent;
pub mod season;
mod types;

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/recent", api = recent::RecentApi),
        (path = "/season", api = season::SeasonApi),
    ),
)]
pub(crate) struct StatsApi;