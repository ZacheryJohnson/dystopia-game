// ZJ-TODO: old APIs - migrate to new
pub mod world_old;
// ZJ-TODO: end old APIs

pub mod game;
pub mod schedule;
pub mod stats;
pub mod world;
pub mod vote;

use std::cell::LazyCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use chrono::{DateTime, Utc};
use dys_simulation::game::Game;
use dys_world::schedule::calendar::Date;
use dys_world::world::World;

use sqlx::{Execute, MySql, QueryBuilder};
use utoipa::OpenApi;
use dys_datastore_mysql::datastore::MySqlDatastore;
use dys_datastore_mysql::execute_query;
use dys_datastore_mysql::query::MySqlQuery;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyDatastore};
use dys_nats::error::NatsError;
use dys_stat::combatant_statline::CombatantStatline;
use dys_world::combatant::instance::EffectDuration;
use dys_world::games::instance::GameInstanceId;
use dys_world::proposal::ProposalEffect;
use dys_world::season::season::Season;
use dys_world::team::instance::TeamInstance;

pub use ::async_nats;
pub use ::tower;
use crate::game::types::GameSummary;
use crate::world_old::InsertGameLogQuery;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/game", api = game::GameApi),
        (path = "/schedule", api = schedule::ScheduleApi),
        (path = "/stats", api = stats::StatsApi),
        (path = "/vote", api = vote::VoteApi),
        (path = "/world", api = world::WorldApi),
    ),
    servers(
        (url = "/api"),
    ),
)]
pub struct DirectorApi {
    endpoints: LazyCell<Vec<Box<dyn tower::Service<
        async_nats::Message,
        Error=NatsError,
        Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
        Response=Bytes,
    > + Send + 'static>>>,
    _app_state: AppState,
}

impl DirectorApi {
    pub fn register_endpoint(
        &mut self,
        endpoint: Box<dyn tower::Service<
            async_nats::Message,
            Error=NatsError,
            Future=Pin<Box<dyn Future<Output = Result<Bytes, NatsError>> + Send>>,
            Response=Bytes,
        > + Send + 'static>,
    ) {
        self.endpoints.push(endpoint);
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub game_world: Arc<Mutex<World>>,
    pub season: Arc<Mutex<Season>>,
    pub current_date: Arc<Mutex<Date>>,
    pub first_game_time_utc: Arc<Mutex<DateTime<Utc>>>,
    pub valkey: Arc<Mutex<ValkeyDatastore>>,
    pub mysql: Arc<Mutex<MySqlDatastore>>,
}

#[tracing::instrument(skip_all)]
async fn simulate_matches(app_state: AppState) -> Vec<(GameSummary, Bytes)> {
    let proposals = {
        let mut valkey_connection = app_state.valkey.lock().unwrap().connection();
        let proposal_jsons: Vec<String> = valkey_connection.hvals(
            "env:dev:proposals:latest"
        ).await.unwrap();

        proposal_jsons
            .iter()
            .map(|proposal_str| serde_json::from_str(proposal_str).unwrap())
            .collect::<Vec<dys_world::proposal::Proposal>>()
    };

    let current_date = app_state.current_date.lock().unwrap().to_owned();
    let game_instances = app_state.season.lock().unwrap().games_on_date(&current_date);

    // Simulate games
    let mut game_results = vec![];
    for game_instance in game_instances {
        let game_instance = game_instance.upgrade().unwrap().lock().unwrap().to_owned();
        let away_team_name = game_instance.away_team.lock().unwrap().name.clone();
        let home_team_name = game_instance.home_team.lock().unwrap().name.clone();

        tracing::info!("Simulating {} vs {} on {:?}", away_team_name, home_team_name, current_date);

        let apply_most_voted_option = |
            votes: Vec<String>,
            proposal: &dys_world::proposal::Proposal,
            team: Arc<Mutex<TeamInstance>>,
        | {
            let mut most_voted_option: (Option<u64>, u32) = (None, 0);
            for option_and_votes_str in votes.chunks(2) {
                let option_str = option_and_votes_str.first();
                let option_id = option_str.unwrap().split(":").collect::<Vec<_>>()[1].parse::<u64>().unwrap();
                let vote_count = option_and_votes_str.get(1).unwrap().parse::<u32>().unwrap();

                if vote_count > most_voted_option.1 {
                    most_voted_option = (Some(option_id), vote_count);
                }
            }

            if let Some(option_id) = most_voted_option.0 {
                let chosen_option = proposal
                    .options
                    .iter()
                    .find(|option| option.id == option_id)
                    .unwrap();

                for effect in &chosen_option.effects {
                    match effect {
                        ProposalEffect::CombatantTemporaryAttributeBonus { combatant_instance_id, attribute_instance_bonus } => {
                            let team = team.lock().unwrap();
                            let mut target_combatant = team
                                .combatants
                                .iter()
                                .find(|com| com.lock().unwrap().id == *combatant_instance_id)
                                .unwrap()
                                .lock()
                                .unwrap();

                            tracing::info!(
                                "Applying bonus {:?} to combatant instance ID {}",
                                attribute_instance_bonus,
                                combatant_instance_id
                            );

                            target_combatant.apply_effect(
                                attribute_instance_bonus.clone(),
                                EffectDuration::NumberOfMatches(1),
                            );
                        }
                    }
                }
            }
        };

        // ZJ-TODO: use IDs instead of string comparison
        if let Some(away_team_proposal) = proposals.iter().find(|prop| prop.name.contains(&away_team_name)) {
            let mut valkey_connection = app_state.valkey.lock().unwrap().connection();

            let away_team_proposal_id = away_team_proposal.id;
            let away_team_proposal_votes: Vec<String> = valkey_connection.hgetall(
                format!("env:dev:votes:proposal:{away_team_proposal_id}"),
            ).await.unwrap();

            apply_most_voted_option(
                away_team_proposal_votes,
                away_team_proposal,
                game_instance.away_team.clone()
            );

            // ZJ-TODO: don't delete, just archive
            let _: u32 = valkey_connection
                .del(format!("env:dev:votes:proposal:{away_team_proposal_id}"))
                .await
                .unwrap();
        }

        if let Some(home_team_proposal) = proposals.iter().find(|prop| prop.name.contains(&home_team_name)) {
            let mut valkey_connection = app_state.valkey.lock().unwrap().connection();

            let home_team_proposal_id = home_team_proposal.id;
            let home_team_proposal_votes: Vec<String> = valkey_connection.hgetall(
                format!("env:dev:votes:proposal:{home_team_proposal_id}"),
            ).await.unwrap();

            apply_most_voted_option(home_team_proposal_votes, home_team_proposal, game_instance.home_team.clone());

            // ZJ-TODO: don't delete, just archive
            let _: u32 = valkey_connection
                .del(format!("env:dev:votes:proposal:{home_team_proposal_id}"))
                .await
                .unwrap();
        }

        let game_id = game_instance.game_id.to_owned();
        let game = Game { game_instance };
        let game_log = game.simulate();

        struct InsertGameStatlines<'q> {
            game_id: GameInstanceId,
            statlines: Vec<CombatantStatline>,
            query_builder: QueryBuilder<'q, MySql>,
        }

        impl<'q> Debug for InsertGameStatlines<'q> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f
                    .debug_struct("InsertGameStatlines")
                    .field("game_id", &self.game_id)
                    .field("statlines", &self.statlines)
                    .finish()
            }
        }

        impl<'q> InsertGameStatlines<'q> {
            pub fn new(game_id: GameInstanceId, statlines: Vec<CombatantStatline>) -> Self {
                InsertGameStatlines {
                    game_id,
                    statlines,
                    query_builder: QueryBuilder::new(
                        "INSERT INTO game_statline (game_id, combatant_id, points, balls_thrown, throws_hit, combatants_shoved) "
                    ),
                }
            }
        }

        impl<'q> MySqlQuery for InsertGameStatlines<'q> {
            fn query(&mut self) -> impl Execute<'_, MySql> {
                self.query_builder.push_values(&self.statlines, |mut builder, statline| {
                    builder
                        .push_bind(self.game_id)
                        .push_bind(statline.combatant_id)
                        .push_bind(statline.points_scored)
                        .push_bind(statline.balls_thrown)
                        .push_bind(statline.throws_hit)
                        .push_bind(statline.combatants_shoved);
                }).build()
            }
        }

        let combatant_statlines = CombatantStatline::from_game_log(&game_log);
        execute_query!(app_state.mysql.clone(), InsertGameStatlines::new(
            game_id,
            combatant_statlines,
        ));

        let serialized_game_log = postcard::to_allocvec(&game_log)
            .expect("failed to serialize game log");

        {
            execute_query!(app_state.mysql.clone(), InsertGameLogQuery {
                game_id: game.game_instance.game_id,
                serialized_game_log: serialized_game_log.clone(),
            });
        }

        let all_combatants = [
            game.game_instance.home_team.lock().unwrap().combatants.clone().as_slice(),
            game.game_instance.away_team.lock().unwrap().combatants.clone().as_slice()].concat();

        for combatant in all_combatants {
            combatant.lock().unwrap().tick_effects();
        }

        let away_team_score = game_log.away_score() as u32;
        let home_team_score = game_log.home_score() as u32;
        let home_win = home_team_score > away_team_score;

        let mut valkey_connection = app_state.valkey.lock().unwrap().connection();
        let _: i32 = valkey_connection.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", &away_team_name),
            if home_win { "losses" } else { "wins" },
            1
        ).await.unwrap();

        let _: i32 = valkey_connection.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", &home_team_name),
            if home_win { "wins" } else { "losses" },
            1
        ).await.unwrap();

        let mut records = HashMap::from([
            (&away_team_name, String::new()),
            (&home_team_name, String::new()),
        ]);

        for (team_name, record) in &mut records {
            let current_record: Vec<String> = valkey_connection.hgetall(
                format!("env:dev:season:record:team:{}", &team_name)
            ).await.unwrap();

            assert_eq!(record.len() % 2, 0);
            let current_record = current_record
                .chunks(2)
                .map(|vals| (vals[0].to_owned(), vals[1].parse::<i32>().unwrap()))
                .collect::<HashMap<_, _>>();

            *record = format!(
                "{}-{}",
                current_record.get(&String::from("wins")).unwrap_or(&0),
                current_record.get(&String::from("losses")).unwrap_or(&0),
            );
        }

        let home_team_record = records.get(&home_team_name).unwrap().clone();
        let away_team_record = records.get(&away_team_name).unwrap().clone();

        game_results.push((GameSummary {
            game_id,
            away_team_name,
            home_team_name,
            away_team_score,
            home_team_score,
            date: current_date.as_iso8601(),
            home_team_record,
            away_team_record,
        }, serialized_game_log.into()));
    }

    *app_state.current_date.lock().unwrap() = Date::new(
        current_date.month(),
        current_date.day() + 1,
        current_date.year()
    );

    game_results
}

#[tracing::instrument(skip_all)]
pub async fn run_simulation(world_state: AppState) {
    let game_summary = simulate_matches(world_state.clone()).await;

    let mut latest_ids = vec![];
    let mut valkey = world_state.valkey.lock().unwrap().connection();

    for (summary, serialized_game_log) in game_summary {
        latest_ids.push(summary.game_id);

        let game_summary_json = serde_json::to_string(&summary).unwrap();
        let _: i32 = valkey.hset(
            format!("env:dev:game.results:id:{}", summary.game_id),
            "summary",
            game_summary_json,
        ).await.unwrap();

        let _: i32 = valkey.hset(
            format!("env:dev:game.results:id:{}", summary.game_id),
            "game_log",
            serialized_game_log.as_ref(),
        ).await.unwrap();

        let _: i32 = valkey.expire(
            format!("env:dev:game.results:id:{}", summary.game_id),
            60 * 60 * 2 // 2 hours
        ).await.unwrap();
    }

    let _: u32 = valkey.lpush(
        "env:dev:game.results:latest",
        latest_ids,
    ).await.unwrap();

    let _: String = valkey.ltrim(
        "env:dev:game.results:latest",
        0,
        10,
    ).await.unwrap();
}