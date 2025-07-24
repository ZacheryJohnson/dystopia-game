use sqlx::{Connection, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlRow};
use dys_datastore::datastore::Datastore;
use crate::query::MySqlQuery;

#[derive(Debug)]
pub struct MySqlDatastore {
    connection: MySqlPool,
}

impl MySqlDatastore {
    pub fn prepare_query(&self) -> MySqlDatastoreQueryWrapper {
        MySqlDatastoreQueryWrapper {
            connection: self.connection.clone(),
        }
    }
}

pub struct MySqlDatastoreQueryWrapper {
    connection: MySqlPool,
}

impl MySqlDatastoreQueryWrapper {
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, query: impl MySqlQuery) {
        query
            .query()
            .execute(&self.connection)
            .await
            .unwrap();
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch(&self, query: impl MySqlQuery) -> MySqlRow {
        query
            .query()
            .fetch_one(&self.connection)
            .await
            .unwrap()
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_all(&self, query: impl MySqlQuery) -> Vec<MySqlRow> {
        query
            .query()
            .fetch_all(&self.connection)
            .await
            .unwrap()
    }

}

impl Datastore for MySqlDatastore {
    type DatastoreConfig = MySqlConnectOptions;

    async fn connect(config: Self::DatastoreConfig) -> Result<Self, ()> {
        match MySqlPool::connect_with(config).await {
            Ok(connection) => Ok(Self { connection }),
            Err(err) => panic!("Failed to connect to MySQL database: {err}"),
        }
    }

    async fn check_connection(&mut self) -> bool {
        self.connection.acquire().await.unwrap().ping().await.is_ok()
    }
}