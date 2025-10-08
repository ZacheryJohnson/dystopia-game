use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub(super) struct Statline {
    pub points: i64,
    pub throws: u64,
    pub hits: u64,
    pub shoves: u64,
}