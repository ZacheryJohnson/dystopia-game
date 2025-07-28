use std::sync::{Arc, Mutex};
use rand::prelude::StdRng;
use rand::SeedableRng;
use sqlx::MySql;
use sqlx::mysql::MySqlArguments;
use sqlx::query::Query;
use dys_datastore_mysql::datastore::MySqlDatastore;
use dys_datastore_mysql::query::MySqlQuery;
use dys_world::games::instance::GameInstanceId;
use dys_world::season::season::Season;
use dys_world::team::instance::TeamInstanceId;
use dys_world::world::World;

#[derive(Debug)]
pub struct InsertSeasonQuery {
    pub season_id: u32,
}

impl MySqlQuery for InsertSeasonQuery {
    fn query(&self) -> Query<MySql, MySqlArguments> {
        sqlx::query!("
            INSERT IGNORE INTO season(season_id)
            VALUES (?)
        ",
            self.season_id
        )
    }
}

#[derive(Debug)]
pub struct InsertCorporationQuery {
    pub corp_id: TeamInstanceId,
    pub corp_name: String,
}

impl MySqlQuery for InsertCorporationQuery {
    fn query(&self) -> Query<MySql, MySqlArguments> {
        sqlx::query!(
            "INSERT INTO corporation(corp_id, name) VALUES (?, ?)",
            self.corp_id,
            self.corp_name,
        )
    }
}

#[derive(Debug)]
pub struct InsertGameLogQuery {
    pub game_id: GameInstanceId,
    pub serialized_game_log: Vec<u8>,
}

impl MySqlQuery for InsertGameLogQuery {
    fn query(&self) -> Query<MySql, MySqlArguments> {
        sqlx::query!("
            INSERT INTO game_results(game_id, serialized_results)
            VALUES (?, ?)",
            self.game_id,
            self.serialized_game_log.as_slice()
        )
    }
}

#[derive(Debug)]
pub struct InsertGameQuery {
    pub game_id: GameInstanceId,
    pub season_id: u32,
    pub team_1: TeamInstanceId,
    pub team_2: TeamInstanceId,
}

impl MySqlQuery for InsertGameQuery {
    fn query(&self) -> Query<MySql, MySqlArguments> {
        sqlx::query!("
            INSERT INTO game(game_id, season_id, team_1, team_2)
            VALUES (?, ?, ?, ?)",
            self.game_id,
            self.season_id,
            self.team_1,
            self.team_2,
        )
    }
}

pub async fn generate_world() -> (Arc<Mutex<World>>, Season) {
    let generator = dys_world::generator::Generator::new();
    let world = generator.generate_world(&mut StdRng::from_os_rng());

    let season = generator.generate_season(&mut StdRng::from_os_rng(), &world);

    (Arc::new(Mutex::new(world)), season)
}

#[tracing::instrument(skip_all)]
pub async fn save_world(
    mysql: Arc<Mutex<MySqlDatastore>>,
    game_world: Arc<Mutex<World>>,
    season: &Season,
) {
    mysql
        .lock()
        .unwrap()
        .prepare_query()
        .execute(InsertSeasonQuery { season_id: 1 })
        .await;

    for team in game_world.lock().unwrap().teams.to_owned() {
        let team = team.lock().unwrap();

        mysql.lock().unwrap().prepare_query().execute(InsertCorporationQuery {
            corp_id: team.id,
            corp_name: team.name.clone(),
        }).await;
    }

    for series in season.series() {
        for game in &series.games() {
            let game = game.lock().unwrap();

            mysql.lock().unwrap().prepare_query().execute(InsertGameQuery {
                game_id: game.game_id,
                season_id: 1, // ZJ-TODO
                team_1: game.away_team.lock().unwrap().id,
                team_2: game.home_team.lock().unwrap().id,
            }).await;
        }
    }
}