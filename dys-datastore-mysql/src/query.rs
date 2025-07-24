use std::fmt::Debug;
use sqlx::MySql;
use sqlx::mysql::MySqlArguments;
use sqlx::query::Query;

pub trait MySqlQuery: Debug + Sized {
    fn query(&self) -> Query<MySql, MySqlArguments>;
}