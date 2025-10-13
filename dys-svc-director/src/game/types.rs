use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GamePreview {
    pub game_id: u32,
    pub away_team_name: String,
    pub home_team_name: String,
    pub date: String, // ZJ-TODO: ideally use dys_world's date; needs to impl ToSchema
    pub home_team_record: String,
    pub away_team_record: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GameSummary {
    pub game_id: u32,
    pub away_team_name: String,
    pub home_team_name: String,
    pub away_team_score: u32,
    pub home_team_score: u32,
    pub date: String, // ZJ-TODO: ideally use dys_world's date; needs to impl ToSchema
    pub home_team_record: String,
    pub away_team_record: String,
}