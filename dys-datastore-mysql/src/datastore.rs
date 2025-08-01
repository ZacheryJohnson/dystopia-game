use sqlx::{Connection, Executor, MySqlPool};
use sqlx::mysql::{MySqlConnectOptions, MySqlRow};
use dys_datastore::datastore::Datastore;
use crate::query::MySqlQuery;

/// Issues a query against the database, ignoring any results.
/// The query is run immediately, awaiting the future and returning nothing.
#[macro_export]
macro_rules! execute_query {
    ($mysql_arc_mutex:expr, $query:expr) => {{
        let query_wrapper = $mysql_arc_mutex
            .lock()
            .unwrap()
            .prepare_query();

        query_wrapper.execute($query).await
    }};
}

/// Gets a single row from the database given a query.
/// The query is run immediately, awaiting the future and returning the row.
#[macro_export]
macro_rules! fetch_query {
    ($mysql_arc_mutex:expr, $query:expr) => {{
        let query_wrapper = $mysql_arc_mutex
            .lock()
            .unwrap()
            .prepare_query();

        query_wrapper.fetch($query).await
    }};
}

/// Gets all rows from the database given a query.
/// The query is run immediately, awaiting the future and returning the row.
#[macro_export]
macro_rules! fetch_all_query {
    ($mysql_arc_mutex:expr, $query:expr) => {{
        let query_wrapper = $mysql_arc_mutex
            .lock()
            .unwrap()
            .prepare_query();

        query_wrapper.fetch_all($query).await
    }};
}

#[derive(Clone, Debug)]
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
        self
            .connection
            .execute(query.query())
            .await
            .unwrap();
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch(&self, query: impl MySqlQuery) -> Option<MySqlRow> {
        self
            .connection
            .fetch_optional(query.query())
            .await
            .unwrap()
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_all(&self, query: impl MySqlQuery) -> Vec<MySqlRow> {
        self
            .connection
            .fetch_all(query.query())
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

#[cfg(all(test, feature = "database-tests"))]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use lazy_static::{lazy_static};
    use sqlx::{Execute, MySql, Row};
    use sqlx::mysql::MySqlConnectOptions;
    use super::{Datastore, MySqlDatastore, MySqlQuery};

    lazy_static! {
        static ref DATASTORE: Arc<Mutex<Option<MySqlDatastore>>> = Arc::new(Mutex::new(None));
    }

    async fn get_database_connection() -> Arc<Mutex<MySqlDatastore>> {
        let maybe_datastore = {
            DATASTORE.lock().unwrap().to_owned()
        };

        // If we've initialized the database pool already, return it
        if let Some(datastore) = maybe_datastore {
            return Arc::new(Mutex::new(datastore));
        }

        // We haven't initialized the database pool yet, so create one
        // We can't change the default database connection timeout,
        // so instead rely on Tokio to timeout more quickly than the default 30 seconds.
        let datastore = tokio::time::timeout(Duration::from_secs(2), async move {
            MySqlDatastore::connect(MySqlConnectOptions::new()
                .host(&std::env::var("MYSQL_HOST").unwrap_or(String::from("127.0.0.1")))
                .username(&std::env::var("MYSQL_USER").unwrap_or(String::from("default")))
                .password(&std::env::var("MYSQL_PASS").unwrap_or(String::from("")))
                .port(std::env::var("MYSQL_PORT").unwrap_or(String::from("3306")).parse::<u16>().unwrap())
            ).await.expect("failed to connect to MySQL database")
        }).await.expect("timeout connecting to MySQL database");

        let mut static_datastore = DATASTORE.lock().unwrap();
        Arc::new(Mutex::new(static_datastore.insert(datastore).to_owned()))
    }

    #[tokio::test]
    async fn test_execute_query_macro() {
        let mysql = get_database_connection().await;

        #[derive(Debug)]
        struct TestQuery {
            value: i64
        }
        impl MySqlQuery for TestQuery {
            fn query(&self) -> impl Execute<MySql> {
                sqlx::query!("SELECT ? as col", self.value)
            }
        }

        // Macro param = existing variable
        let test_query = TestQuery { value: 3 };
        execute_query!(mysql, test_query);

        // Macro param = newly created temporary
        execute_query!(mysql, TestQuery { value: 1 });
    }

    #[tokio::test]
    async fn test_fetch_query_macro() {
        let mysql = get_database_connection().await;

        #[derive(Debug)]
        struct TestQuery {
            value: i64,
        }
        impl MySqlQuery for TestQuery {
            fn query(&self) -> impl Execute<MySql> {
                sqlx::query!("SELECT ? as col", self.value)
            }
        }

        let expected_value = 3;

        // Macro param = existing variable
        let test_query = TestQuery { value: expected_value };
        let result = fetch_query!(mysql, test_query).unwrap();
        let val: i64 = result.get("col");
        assert_eq!(expected_value, val);

        // Macro param = newly created temporary
        let result = fetch_query!(mysql, TestQuery { value: expected_value }).unwrap();
        let val: i64 = result.get("col");
        assert_eq!(expected_value, val);
    }

    #[tokio::test]
    async fn test_fetch_returning_nothing() {
        let mysql = get_database_connection().await;

        #[derive(Debug)]
        struct TestQuery {
            value: i64,
        }
        impl MySqlQuery for TestQuery {
            fn query(&self) -> impl Execute<MySql> {
                sqlx::query!("SELECT ? as col WHERE 1 = 2", self.value)
            }
        }

        let expected_value = 3;

        // Macro param = existing variable
        let test_query = TestQuery { value: expected_value };
        let result = fetch_query!(mysql, test_query);
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_fetch_all_query_macro() {
        let mysql = get_database_connection().await;

        #[derive(Debug)]
        struct TestQuery {
            value_1: i64,
            value_2: i64,
        }
        impl MySqlQuery for TestQuery {
            fn query(&self) -> impl Execute<MySql> {
                sqlx::query!("SELECT ? as col UNION SELECT ? as col", self.value_1, self.value_2)
            }
        }

        let expected_value_1 = 1;
        let expected_value_2 = 2;

        let expected_values = vec![expected_value_1, expected_value_2];

        let test_query = TestQuery { value_1: expected_value_1, value_2: expected_value_2 };
        // Macro param = existing variable
        let result = fetch_all_query!(mysql, test_query);
        assert_eq!(2, result.len());
        for (row, expected_value) in result.iter().zip(expected_values.iter()) {
            let val: i64 = row.get("col");
            assert_eq!(*expected_value, val);
        }

        // Macro param = newly created temporary
        let result = fetch_all_query!(mysql, TestQuery {
            value_1: expected_value_1,
            value_2: expected_value_2
        });
        assert_eq!(2, result.len());
        for (row, expected_value) in result.iter().zip(expected_values.iter()) {
            let val: i64 = row.get("col");
            assert_eq!(*expected_value, val);
        }
    }

    #[tokio::test]
    async fn test_fetch_all_returns_empty_vec() {
        let mysql = get_database_connection().await;

        #[derive(Debug)]
        struct TestQuery {
            value_1: i64,
            value_2: i64,
        }
        impl MySqlQuery for TestQuery {
            fn query(&self) -> impl Execute<MySql> {
                sqlx::query!("SELECT ? as col WHERE 1 = 2 UNION SELECT ? as col WHERE 1 = 2",
                    self.value_1,
                    self.value_2
                )
            }
        }

        let test_query = TestQuery { value_1: 1, value_2: 2 };
        let result = fetch_all_query!(mysql, test_query);
        assert_eq!(0, result.len());
    }
}