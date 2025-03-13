mod match_result;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use bytes::Bytes;
use dys_observability::logger::LoggerOptions;
use dys_simulation::game::Game;
use dys_world::schedule::calendar::{Date, Month};
use dys_world::world::World;
use serde::Serialize;

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use dys_datastore::datastore::Datastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_nats::error::NatsError;
use dys_nats::router::NatsRouter;
use dys_protocol::nats::match_results::match_response::MatchSummary;
use dys_protocol::nats::match_results::summary_svc::{GetGameLogRpcServer, MatchesRpcServer};
use dys_protocol::nats::vote::{GetProposalsRequest, GetProposalsResponse, Proposal, ProposalOption, VoteOnProposalRequest, VoteOnProposalResponse};
use dys_protocol::nats::vote::vote_svc::{GetProposalsRpcServer, VoteOnProposalRpcServer};
use dys_protocol::nats::world::{GetSeasonRequest, GetSeasonResponse, WorldStateRequest, WorldStateResponse};
use dys_protocol::nats::world::schedule_svc::GetSeasonRpcServer;
use dys_protocol::nats::world::world_svc::WorldStateRpcServer;
use dys_world::combatant::instance::CombatantInstance;
use dys_world::schedule::calendar::Month::Arguscorp;
use dys_world::schedule::season::Season;
use dys_world::schedule::series::SeriesType;
use crate::match_result::{get_game_log, get_summaries};

#[derive(Clone, Debug)]
struct AppState {
    game_world: Arc<Mutex<World>>,
    season: Arc<Mutex<Season>>,
    current_date: Arc<Mutex<Date>>,
    valkey: ValkeyDatastore,
    nats: async_nats::Client,
}

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "director".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");

    let (game_world, season) = {
        let generator = dys_world::generator::Generator::new();
        let world = generator.generate_world(&mut StdRng::from_entropy());

        let season = generator.generate_season(&mut StdRng::from_entropy(), &world);

        (Arc::new(Mutex::new(world)), season)
    };

    let valkey_config = ValkeyConfig::new(
        std::env::var("VALKEY_USER").unwrap_or(String::from("default")),
        std::env::var("VALKEY_PASS").unwrap_or(String::from("")),
        std::env::var("VALKEY_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("VALKEY_PORT").unwrap_or(String::from("6379")).parse::<u16>().unwrap()
    );

    let nats_server_str = format!(
        "{}:{}",
        std::env::var("NATS_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("NATS_PORT").unwrap_or(String::from("4222")).parse::<u16>().unwrap(),
    );

    let nats_client = async_nats::ConnectOptions::new()
        .token(std::env::var("NATS_TOKEN").unwrap_or(String::from("replaceme")))
        .connect(nats_server_str)
        .await
        .expect("failed to connect to NATS server");

    let app_state = AppState {
        game_world: game_world.clone(),
        season: Arc::new(Mutex::new(season)),
        current_date: Arc::new(Mutex::new(Date(
            Arguscorp, 1, 10000
        ))),
        valkey: *ValkeyDatastore::connect(valkey_config).await.unwrap(),
        nats: nats_client,
    };

    let app_state_thread_copy = app_state.clone();
    const SLEEP_DURATION: Duration = Duration::from_secs(5 * 60);

    tokio::task::spawn(async move {
        loop {
            tracing::info!("Executing simulations...");
            run_simulation(app_state_thread_copy.clone()).await;

            let mut app_state = app_state_thread_copy.clone();
            let mut valkey = app_state.valkey.connection();
            let world = app_state.game_world.lock().unwrap().to_owned();

            tracing::info!("Saving world state in valkey...");
            let _: i32 = valkey.hset(
                "env:dev:world",
                "data",
                serde_json::to_string(&world).unwrap(),
            ).await.unwrap();

            let _: i32 = valkey.expire(
                "env:dev:world",
                450,
            ).await.unwrap();

            // Sleep before simulating more matches
            tracing::info!("Sleeping for {} seconds before simulating more matches...", SLEEP_DURATION.as_secs());
            tokio::time::sleep(SLEEP_DURATION).await;
        }
    });

    let nats = NatsRouter::new()
        .await
        .service(MatchesRpcServer::with_handler_and_state(get_summaries, app_state.clone()))
        .service(GetGameLogRpcServer::with_handler_and_state(get_game_log, app_state.clone()))
        .service(GetSeasonRpcServer::with_handler_and_state(get_season, app_state.clone()))
        .service(WorldStateRpcServer::with_handler_and_state(get_world_state, app_state.clone()))
        .service(GetProposalsRpcServer::with_handler_and_state(get_voting_proposals, app_state.clone()))
        .service(VoteOnProposalRpcServer::with_handler_and_state(submit_vote, app_state.clone()));

    nats.run().await;
}

fn simulate_matches(app_state: AppState) -> Vec<(MatchSummary, Bytes)> {
    let current_date = app_state.current_date.lock().unwrap().to_owned();
    let match_instances = app_state.season.lock().unwrap().matches_on_date(&current_date);

    // Simulate matches
    let mut match_results = vec![];
    for match_instance in match_instances {
        let match_instance = match_instance.lock().unwrap().to_owned();
        let away_team_name = match_instance.away_team.lock().unwrap().name.clone();
        let home_team_name = match_instance.home_team.lock().unwrap().name.clone();

        tracing::info!("Simulating {} vs {} on {:?}", away_team_name, home_team_name, current_date);

        let match_id = match_instance.match_id.to_owned();
        let game = Game { match_instance };
        let game_log = game.simulate();

        match_results.push((MatchSummary {
            match_id,
            away_team_name,
            home_team_name,
            away_team_score: game_log.away_score() as u32,
            home_team_score: game_log.home_score() as u32,
            date: Some(dys_protocol::nats::common::Date {
                year: current_date.2,
                month: current_date.0.to_owned() as i32 + 1, // ZJ-TODO: yuck
                day: current_date.1,
            }),
            home_team_record: String::new(),
            away_team_record: String::new(),
        }, postcard::to_allocvec(&game_log).expect("failed to serialize game log").into()));
    }

    *app_state.current_date.lock().unwrap() = Date(
        current_date.0,
        current_date.1 + 1,
        current_date.2
    );

    match_results
}

#[tracing::instrument(skip_all)]
async fn run_simulation(mut world_state: AppState) {
    let match_summary = simulate_matches(world_state.clone());

    let mut latest_ids = vec![];
    let mut valkey = world_state.valkey.connection();
    for (mut summary, serialized_game_log) in match_summary {
        latest_ids.push(summary.match_id);

        let home_win = summary.home_team_score > summary.away_team_score;

        let _: i32 = valkey.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", summary.away_team_name),
            if home_win { "losses" } else { "wins" },
            1
        ).await.unwrap();

        let _: i32 = valkey.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", summary.home_team_name),
            if home_win { "wins" } else { "losses" },
            1
        ).await.unwrap();

        let away_team_record: Vec<String> = valkey.hgetall(
            format!("env:dev:season:record:team:{}", summary.away_team_name)
        ).await.unwrap();
        assert_eq!(away_team_record.len() % 2, 0);
        let away_team_record = away_team_record
            .chunks(2)
            .map(|vals| (vals[0].to_owned(), vals[1].parse::<i32>().unwrap()))
            .collect::<HashMap<_, _>>();
        let away_team_record = format!(
            "{}-{}",
            away_team_record.get(&String::from("wins")).unwrap_or(&0),
            away_team_record.get(&String::from("losses")).unwrap_or(&0),
        );

        let home_team_record: Vec<String> = valkey.hgetall(
            format!("env:dev:season:record:team:{}", summary.home_team_name)
        ).await.unwrap();
        assert_eq!(home_team_record.len() % 2, 0);
        let home_team_record = home_team_record
            .chunks(2)
            .map(|vals| (vals[0].to_owned(), vals[1].parse::<i32>().unwrap()))
            .collect::<HashMap<_, _>>();
        let home_team_record = format!(
            "{}-{}",
            home_team_record.get(&String::from("wins")).unwrap_or(&0),
            home_team_record.get(&String::from("losses")).unwrap_or(&0),
        );

        summary.away_team_record = away_team_record;
        summary.home_team_record = home_team_record;

        let match_summary_json = serde_json::to_string(&summary).unwrap();
        let _: i32 = valkey.hset(
            format!("env:dev:match.results:id:{}", summary.match_id),
            "summary",
            match_summary_json,
        ).await.unwrap();

        let _: i32 = valkey.hset(
            format!("env:dev:match.results:id:{}", summary.match_id),
            "game_log",
            serialized_game_log.as_ref(),
        ).await.unwrap();

        let _: i32 = valkey.expire(
            format!("env:dev:match.results:id:{}", summary.match_id),
            60 * 60 // 1 hour
        ).await.unwrap();
    }

    let _: u32 = valkey.lpush(
        "env:dev:match.results:latest",
        latest_ids,
    ).await.unwrap();

    let _: String = valkey.ltrim(
        "env:dev:match.results:latest",
        0,
        10,
    ).await.unwrap();
}

#[tracing::instrument(skip(app_state))]
async fn get_season(
    _: GetSeasonRequest,
    app_state: AppState,
) -> Result<GetSeasonResponse, NatsError> {
    let season = app_state.season.lock().unwrap();

    let proto_series = season
        .all_series
        .iter()
        .map(|rs_series| {
            dys_protocol::nats::world::Series {
                matches: rs_series
                    .matches
                    .iter()
                    .map(|rs_match| {
                        let rs_match = rs_match.lock().unwrap();
                        let x = dys_protocol::nats::world::MatchInstance {
                            match_id: rs_match.match_id.to_owned(),
                            home_team_id: rs_match.home_team.lock().unwrap().id.to_owned(),
                            away_team_id: rs_match.away_team.lock().unwrap().id.to_owned(),
                            arena_id: 0,
                            // arena_id: rs_match.arena.lock().unwrap().id.to_owned(),
                            date: Some(dys_protocol::nats::common::Date {
                                year: rs_match.date.2,
                                month: 1, // ZJ-TODO: arguscorp
                                day: rs_match.date.1,
                            })
                        };
                        x
                    })
                    .collect::<Vec<_>>(),
                series_type: if matches!(rs_series.series_type, SeriesType::Normal) {
                    dys_protocol::nats::world::series::SeriesType::Normal
                } else {
                    dys_protocol::nats::world::series::SeriesType::FirstTo
                } as i32,
                series_type_payload: vec![], // ZJ-TODO
            }
        })
        .collect::<Vec<_>>();

    let current_date = app_state.current_date.lock().unwrap().to_owned();

    Ok(GetSeasonResponse {
        season_id: 1,
        current_date: Some(dys_protocol::nats::common::Date {
            year: current_date.2,
            month: 1, // ZJ-TODO: arguscorp
            day: current_date.1,
        }),
        all_series: proto_series,
    })
}

#[tracing::instrument(skip(app_state))]
async fn get_world_state(
    _: WorldStateRequest,
    mut app_state: AppState,
) -> Result<WorldStateResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();
    let response_data: String = valkey.hget("env:dev:world", "data").await.unwrap();

    Ok(WorldStateResponse {
        world_state_json: response_data.into_bytes(),
    })
}

#[tracing::instrument(skip(app_state))]
async fn get_voting_proposals(
    _: GetProposalsRequest,
    mut app_state: AppState,
) -> Result<GetProposalsResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();
    let response_data: String = valkey.hget("env:dev:world", "data").await.unwrap();
    let world: World = serde_json::from_str(&response_data).unwrap();
    let team = world.teams.choose(&mut rand::thread_rng()).unwrap();

    let team_instance = team.lock().unwrap();
    let team_name = team_instance.name.to_owned();

    let combatants = team_instance.combatants.clone().into_iter()
        .take(3)
        .collect::<Vec<Arc<Mutex<CombatantInstance>>>>();
    let combatant_1_name = combatants[0].lock().unwrap().name.to_owned();
    let combatant_2_name = combatants[1].lock().unwrap().name.to_owned();
    let combatant_3_name = combatants[2].lock().unwrap().name.to_owned();

    let response = GetProposalsResponse {
        proposals: vec![
            Proposal {
                proposal_id: 1,
                proposal_name: format!("Supercharge {} Player", team_name),
                proposal_desc: "Pick a combatant to supercharge for a match.".to_string(),
                proposal_options: vec![
                    ProposalOption {
                        option_id: 1,
                        option_name: combatant_1_name,
                        option_desc: "".to_string(),
                    },
                    ProposalOption {
                        option_id: 2,
                        option_name: combatant_2_name,
                        option_desc: "".to_string(),
                    },
                    ProposalOption {
                        option_id: 3,
                        option_name: combatant_3_name,
                        option_desc: "".to_string(),
                    }
                ],
            },
        ],
    };

    Ok(response)
}

#[tracing::instrument(skip(app_state, request))]
async fn submit_vote(
    request: VoteOnProposalRequest,
    mut app_state: AppState,
) -> Result<VoteOnProposalResponse, NatsError> {
    let mut valkey = app_state.valkey.connection();

    let _: i32 = valkey.hincr(
        format!("env:dev:votes:proposal:{}", request.proposal_id),
        format!("option:{}", request.option_id),
        1,
    ).await.unwrap();

    Ok(VoteOnProposalResponse {})
}