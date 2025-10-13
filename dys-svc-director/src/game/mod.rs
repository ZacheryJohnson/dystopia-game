pub mod log;
pub mod summary;
pub mod types;

#[derive(utoipa::OpenApi)]
#[openapi(
    nest(
        (path = "/log", api = log::LogApi),
        (path = "/summary", api = summary::SummaryApi),
    ),
)]
pub(crate) struct GameApi;