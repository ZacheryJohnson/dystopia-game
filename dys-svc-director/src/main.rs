mod match_result;
mod world;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use bytes::Bytes;
use chrono::DateTime;
use dys_observability::logger::LoggerOptions;
use dys_simulation::game::Game;
use dys_world::schedule::calendar::Date;
use dys_world::world::World;

use rand::SeedableRng;
use rand::rngs::StdRng;
use sqlx::mysql::MySqlConnectOptions;
use tokio::time::Instant;
use dys_datastore::datastore::Datastore;
use dys_datastore_mysql::datastore::MySqlDatastore;
use dys_datastore_valkey::datastore::{AsyncCommands, ValkeyConfig, ValkeyDatastore};
use dys_nats::error::NatsError;
use dys_nats::rpc::router::NatsRouter;
use dys_protocol::nats::match_results::match_response::MatchSummary;
use dys_protocol::nats::match_results::summary_svc::{GetGameLogRpcServer, MatchesRpcServer};
use dys_protocol::nats::vote::{GetProposalsRequest, GetProposalsResponse, Proposal, ProposalOption, VoteOnProposalRequest, VoteOnProposalResponse};
use dys_protocol::nats::vote::vote_svc::{GetProposalsRpcServer, VoteOnProposalRpcServer};
use dys_protocol::nats::world::{GetSeasonRequest, GetSeasonResponse, WorldStateRequest, WorldStateResponse};
use dys_protocol::nats::world::schedule_svc::GetSeasonRpcServer;
use dys_protocol::nats::world::world_svc::WorldStateRpcServer;
use dys_world::combatant::instance::EffectDuration;
use dys_world::proposal::ProposalEffect;
use dys_world::schedule::calendar::Month::Arguscorp;
use dys_world::schedule::season::Season;
use dys_world::schedule::series::SeriesType;
use dys_world::team::instance::TeamInstance;
use crate::match_result::{get_game_log, get_summaries};
use crate::world::{generate_world, InsertCorporationQuery, InsertGameLogQuery, InsertGameQuery, InsertSeasonQuery};

#[derive(Clone, Debug)]
struct AppState {
    game_world: Arc<Mutex<World>>,
    season: Arc<Mutex<Season>>,
    current_date: Arc<Mutex<Date>>,
    valkey: Arc<Mutex<ValkeyDatastore>>,
    mysql: Arc<Mutex<MySqlDatastore>>,
}

#[tokio::main]
async fn main() {
    let logger_options = LoggerOptions {
        application_name: "director".to_string(),
        ..Default::default()
    };

    dys_observability::logger::initialize(logger_options);

    tracing::info!("Starting server...");

    let (game_world, season) = generate_world().await;

    let valkey_config = ValkeyConfig::new(
        std::env::var("VALKEY_USER").unwrap_or(String::from("default")),
        std::env::var("VALKEY_PASS").unwrap_or(String::from("")),
        std::env::var("VALKEY_HOST").unwrap_or(String::from("172.18.0.1")),
        std::env::var("VALKEY_PORT").unwrap_or(String::from("6379")).parse::<u16>().unwrap()
    );

    let mysql_config = MySqlConnectOptions::new()
        .host(&std::env::var("MYSQL_HOST").unwrap_or(String::from("127.0.0.1")))
        .username(&std::env::var("MYSQL_USER").unwrap_or(String::from("default")))
        .password(&std::env::var("MYSQL_PASS").unwrap_or(String::from("")))
        .port(std::env::var("MYSQL_PORT").unwrap_or(String::from("3306")).parse::<u16>().unwrap())
        .database(&std::env::var("MYSQL_DATABASE").unwrap_or(String::from("")));

    let mysql = Arc::new(Mutex::new(
        MySqlDatastore::connect(mysql_config).await.unwrap()
    ));

    world::save_world(mysql.clone(), game_world.clone(), &season).await;

    let app_state = AppState {
        game_world: game_world.clone(),
        season: Arc::new(Mutex::new(season)),
        current_date: Arc::new(Mutex::new(Date(
            Arguscorp, 1, 10000
        ))),
        valkey: Arc::new(Mutex::new(
            ValkeyDatastore::connect(valkey_config).await.unwrap()
        )),
        mysql,
    };

    let app_state_thread_copy = app_state.clone();
    tokio::task::spawn(async move {
        loop {
            let app_state = app_state_thread_copy.clone();
            let mut valkey = app_state.valkey.lock().unwrap().connection();
            let world = app_state.game_world.lock().unwrap().to_owned();

            tracing::info!("Saving world state in valkey...");
            let _: i32 = valkey.hset(
                "env:dev:world",
                "data",
                serde_json::to_string(&world).unwrap(),
            ).await.unwrap();

            let _: i32 = valkey.expire(
                "env:dev:world",
                60 * 60, // 1 hour
            ).await.unwrap();

            // Generate new proposals for the upcoming matches
            let generator = dys_world::generator::Generator::new();
            let proposals = generator.generate_proposals(&mut StdRng::from_os_rng(), &world);

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
            let next_time = {
                let season = app_state.season.lock().unwrap();
                let match_times = season.simulation_timings.values().to_owned();
                match_times
                    .filter(|time| time.timestamp() >= chrono::Utc::now().timestamp())
                    .min()
                    .map_or(DateTime::default(), |time| time.to_owned())
            };

            let offset = next_time.timestamp()- chrono::Utc::now().timestamp();
            let instant = Instant::now() + Duration::from_secs(offset as u64);
            tracing::info!("Sleeping until {} before simulating more matches...", next_time.to_rfc3339());
            tokio::time::sleep_until(instant).await;

            tracing::info!("Executing simulations...");
            run_simulation(app_state_thread_copy.clone()).await;
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

#[tracing::instrument(skip_all)]
async fn simulate_matches(app_state: AppState) -> Vec<(MatchSummary, Bytes)> {
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
                match_instance.away_team.clone()
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

            apply_most_voted_option(home_team_proposal_votes, home_team_proposal, match_instance.home_team.clone());

            // ZJ-TODO: don't delete, just archive
            let _: u32 = valkey_connection
                .del(format!("env:dev:votes:proposal:{home_team_proposal_id}"))
                .await
                .unwrap();
        }

        let match_id = match_instance.match_id.to_owned();
        let game = Game { match_instance };
        let game_log = game.simulate();
        let serialized_game_log = postcard::to_allocvec(&game_log)
            .expect("failed to serialize game log");

        {
            let query = app_state.mysql.lock().unwrap().prepare_query();
            query.execute(InsertGameLogQuery {
                game_id: game.match_instance.match_id,
                serialized_game_log: serialized_game_log.clone(),
            }).await;
        }

        let all_combatants = [
            game.match_instance.home_team.lock().unwrap().combatants.clone().as_slice(),
            game.match_instance.away_team.lock().unwrap().combatants.clone().as_slice()].concat();

        for combatant in all_combatants {
            combatant.lock().unwrap().tick_effects();
        }

        match_results.push((MatchSummary {
            match_id: Some(match_id),
            away_team_name: Some(away_team_name),
            home_team_name: Some(home_team_name),
            away_team_score: Some(game_log.away_score() as u32),
            home_team_score: Some(game_log.home_score() as u32),
            date: Some(dys_protocol::nats::common::Date {
                year: current_date.2,
                month: current_date.0.to_owned() as i32 + 1, // ZJ-TODO: yuck
                day: current_date.1,
            }),
            home_team_record: None,
            away_team_record: None,
        }, serialized_game_log.into()));
    }

    *app_state.current_date.lock().unwrap() = Date(
        current_date.0,
        current_date.1 + 1,
        current_date.2
    );

    match_results
}

#[tracing::instrument(skip_all)]
async fn run_simulation(world_state: AppState) {
    let match_summary = simulate_matches(world_state.clone()).await;

    let mut latest_ids = vec![];
    let mut valkey = world_state.valkey.lock().unwrap().connection();

    for (mut summary, serialized_game_log) in match_summary {
        latest_ids.push(summary.match_id);

        let home_win = summary.home_team_score > summary.away_team_score;

        let _: i32 = valkey.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", summary.away_team_name.as_ref().unwrap_or(&String::new())),
            if home_win { "losses" } else { "wins" },
            1
        ).await.unwrap();

        let _: i32 = valkey.hincr(
            // ZJ-TODO: should be team ID
            format!("env:dev:season:record:team:{}", summary.home_team_name.as_ref().unwrap_or(&String::new())),
            if home_win { "wins" } else { "losses" },
            1
        ).await.unwrap();

        let away_team_record: Vec<String> = valkey.hgetall(
            format!("env:dev:season:record:team:{}", summary.away_team_name.as_ref().unwrap_or(&String::new())),
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
            format!("env:dev:season:record:team:{}", summary.home_team_name.as_ref().unwrap_or(&String::new()))
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

        summary.away_team_record = Some(away_team_record);
        summary.home_team_record = Some(home_team_record);

        let match_summary_json = serde_json::to_string(&summary).unwrap();
        let _: i32 = valkey.hset(
            format!("env:dev:match.results:id:{}", summary.match_id.as_ref().unwrap_or(&0)),
            "summary",
            match_summary_json,
        ).await.unwrap();

        let _: i32 = valkey.hset(
            format!("env:dev:match.results:id:{}", summary.match_id.as_ref().unwrap_or(&0)),
            "game_log",
            serialized_game_log.as_ref(),
        ).await.unwrap();

        let _: i32 = valkey.expire(
            format!("env:dev:match.results:id:{}", summary.match_id.as_ref().unwrap_or(&0)),
            60 * 60 * 2 // 2 hours
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
    let proto_series = {
        let season = app_state.season.lock().unwrap();
        season
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
                                match_id: Some(rs_match.match_id.to_owned()),
                                home_team_id: Some(rs_match.home_team.lock().unwrap().id.to_owned()),
                                away_team_id: Some(rs_match.away_team.lock().unwrap().id.to_owned()),
                                arena_id: Some(0),
                                // arena_id: rs_match.arena.lock().unwrap().id.to_owned(),
                                date: Some(dys_protocol::nats::common::Date {
                                    year: rs_match.date.2,
                                    month: 1, // ZJ-TODO: arguscorp
                                    day: rs_match.date.1,
                                }),
                                utc_scheduled_time: Some(season.simulation_timings.get(&rs_match.match_id).unwrap().timestamp() as u64)
                            };
                            x
                        })
                        .collect::<Vec<_>>(),
                    series_type: Some(if matches!(rs_series.series_type, SeriesType::Normal) {
                        dys_protocol::nats::world::series::SeriesType::Normal
                    } else {
                        dys_protocol::nats::world::series::SeriesType::FirstTo
                    } as i32),
                    series_type_payload: None
                }
            })
            .collect::<Vec<_>>()
    };

    let current_date = app_state.current_date.lock().unwrap().to_owned();

    Ok(GetSeasonResponse {
        season_id: Some(1),
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
    app_state: AppState,
) -> Result<WorldStateResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();
    let response_data: String = valkey.hget("env:dev:world", "data").await.unwrap();

    Ok(WorldStateResponse {
        world_state_json: Some(response_data.into_bytes()),
    })
}

#[tracing::instrument(skip(app_state))]
async fn get_voting_proposals(
    _: GetProposalsRequest,
    app_state: AppState,
) -> Result<GetProposalsResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let proposal_jsons: Vec<String> = valkey.hvals(
        "env:dev:proposals:latest"
    ).await.unwrap();

    let proposals = proposal_jsons
        .iter()
        .map(|proposal_str| serde_json::from_str(proposal_str).unwrap())
        .collect::<Vec<dys_world::proposal::Proposal>>();

    // ZJ-TODO: don't marshal just send

    let mut marshalled_proposals = vec![];
    for proposal in proposals {
        let mut marshalled_options = vec![];
        for option in &proposal.options {
            marshalled_options.push(ProposalOption {
                option_id: Some(option.id),
                option_name: Some(option.name.clone()),
                option_desc: Some(option.description.clone()),
            });
        }

        marshalled_proposals.push(Proposal {
            proposal_id: Some(proposal.id),
            proposal_name: Some(proposal.name.clone()),
            proposal_desc: Some(proposal.description.clone()),
            proposal_options: marshalled_options,
        });
    }

    let response = GetProposalsResponse {
        proposals: marshalled_proposals
    };

    Ok(response)
}

#[tracing::instrument(skip_all)]
async fn submit_vote(
    request: VoteOnProposalRequest,
    app_state: AppState,
) -> Result<VoteOnProposalResponse, NatsError> {
    let mut valkey = app_state.valkey.lock().unwrap().connection();

    let _: i32 = valkey.hincr(
        format!("env:dev:votes:proposal:{}", request.proposal_id.unwrap_or_default()),
        format!("option:{}", request.option_id.unwrap_or_default()),
        1,
    ).await.unwrap();

    Ok(VoteOnProposalResponse {})
}