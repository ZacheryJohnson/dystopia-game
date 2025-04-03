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
use dys_nats::connection::make_client;
use dys_nats::error::NatsError;
use dys_nats::rpc::router::NatsRouter;
use dys_protocol::nats::match_results::match_response::MatchSummary;
use dys_protocol::nats::match_results::summary_svc::{GetGameLogRpcServer, MatchesRpcServer};
use dys_protocol::nats::vote::{GetProposalsRequest, GetProposalsResponse, Proposal, ProposalOption, VoteOnProposalRequest, VoteOnProposalResponse};
use dys_protocol::nats::vote::vote_svc::{GetProposalsRpcServer, VoteOnProposalRpcServer};
use dys_protocol::nats::world::{GetSeasonRequest, GetSeasonResponse, WorldStateRequest, WorldStateResponse};
use dys_protocol::nats::world::schedule_svc::GetSeasonRpcServer;
use dys_protocol::nats::world::world_svc::WorldStateRpcServer;
use dys_world::combatant::instance::{CombatantInstance, EffectDuration};
use dys_world::proposal::ProposalEffect;
use dys_world::schedule::calendar::Month::Arguscorp;
use dys_world::schedule::season::Season;
use dys_world::schedule::series::SeriesType;
use dys_world::team::instance::TeamInstance;
use crate::match_result::{get_game_log, get_summaries};

#[derive(Clone, Debug)]
struct AppState {
    game_world: Arc<Mutex<World>>,
    season: Arc<Mutex<Season>>,
    current_date: Arc<Mutex<Date>>,
    valkey: ValkeyDatastore,
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

    let app_state = AppState {
        game_world: game_world.clone(),
        season: Arc::new(Mutex::new(season)),
        current_date: Arc::new(Mutex::new(Date(
            Arguscorp, 1, 10000
        ))),
        valkey: *ValkeyDatastore::connect(valkey_config).await.unwrap(),
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

            // Generate new proposals for the upcoming matches
            let generator = dys_world::generator::Generator::new();
            let proposals = generator.generate_proposals(&mut StdRng::from_entropy(), &world);

            let mut zj_todo_id = 1;
            for proposal in proposals {
                let _: i32 = valkey.hset(
                    "env:dev:proposals:latest",
                    zj_todo_id.to_string(),
                    serde_json::to_string(&proposal).unwrap(),
                ).await.unwrap();

                zj_todo_id += 1;
            }

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

async fn simulate_matches(mut app_state: AppState) -> Vec<(MatchSummary, Bytes)> {
    let proposal_jsons: Vec<String> = app_state.valkey.connection().hvals(
        "env:dev:proposals:latest"
    ).await.unwrap();

    let proposals = proposal_jsons
        .iter()
        .map(|proposal_str| serde_json::from_str(&proposal_str).unwrap())
        .collect::<Vec<dys_world::proposal::Proposal>>();

    let current_date = app_state.current_date.lock().unwrap().to_owned();
    let match_instances = app_state.season.lock().unwrap().matches_on_date(&current_date);

    // Simulate matches
    let mut match_results = vec![];
    for match_instance in match_instances {
        let match_instance = match_instance.lock().unwrap().to_owned();
        let away_team_name = match_instance.away_team.lock().unwrap().name.clone();
        let home_team_name = match_instance.home_team.lock().unwrap().name.clone();

        tracing::info!("Simulating {} vs {} on {:?}", away_team_name, home_team_name, current_date);

        let apply_most_voted_option = |
            votes: Vec<String>,
            proposal: &dys_world::proposal::Proposal,
            team: Arc<Mutex<TeamInstance>>,
        | {
            let mut most_voted_option: (Option<u64>, u32) = (None, 0);
            for option_and_votes_str in votes.chunks(2) {
                let option_str = option_and_votes_str.get(0);
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
            let away_team_proposal_id = away_team_proposal.id;
            let away_team_proposal_votes: Vec<String> = app_state.valkey.connection().hgetall(
                format!("env:dev:votes:proposal:{}", away_team_proposal_id),
            ).await.unwrap();

            apply_most_voted_option(
                away_team_proposal_votes,
                away_team_proposal,
                match_instance.away_team.clone()
            );

            // ZJ-TODO: don't delete, just archive
            let _: u32 = app_state.valkey.connection()
                .del(format!("env:dev:votes:proposal:{}", away_team_proposal_id))
                .await
                .unwrap();
        }

        if let Some(home_team_proposal) = proposals.iter().find(|prop| prop.name.contains(&home_team_name)) {
            let home_team_proposal_id = home_team_proposal.id;
            let home_team_proposal_votes: Vec<String> = app_state.valkey.connection().hgetall(
                format!("env:dev:votes:proposal:{}", home_team_proposal_id),
            ).await.unwrap();

            apply_most_voted_option(home_team_proposal_votes, home_team_proposal, match_instance.home_team.clone());

            // ZJ-TODO: don't delete, just archive
            let _: u32 = app_state.valkey.connection()
                .del(format!("env:dev:votes:proposal:{}", home_team_proposal_id))
                .await
                .unwrap();
        }

        let match_id = match_instance.match_id.to_owned();
        let game = Game { match_instance };
        let game_log = game.simulate();

        let all_combatants = [
            game.match_instance.home_team.lock().unwrap().combatants.clone().as_slice(),
            game.match_instance.away_team.lock().unwrap().combatants.clone().as_slice()].concat();

        for combatant in all_combatants {
            combatant.lock().unwrap().tick_effects();
        }

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
    let match_summary = simulate_matches(world_state.clone()).await;

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

    let proposal_jsons: Vec<String> = valkey.hvals(
        "env:dev:proposals:latest"
    ).await.unwrap();

    let proposals = proposal_jsons
        .iter()
        .map(|proposal_str| serde_json::from_str(&proposal_str).unwrap())
        .collect::<Vec<dys_world::proposal::Proposal>>();

    // ZJ-TODO: don't marshal just send

    let mut marshalled_proposals = vec![];
    for proposal in proposals {
        let mut marshalled_options = vec![];
        for option in &proposal.options {
            marshalled_options.push(ProposalOption {
                option_id: option.id,
                option_name: option.name.clone(),
                option_desc: option.description.clone(),
            });
        }

        marshalled_proposals.push(Proposal {
            proposal_id: proposal.id,
            proposal_name: proposal.name.clone(),
            proposal_desc: proposal.description.clone(),
            proposal_options: marshalled_options,
        });
    }

    let response = GetProposalsResponse {
        proposals: marshalled_proposals
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