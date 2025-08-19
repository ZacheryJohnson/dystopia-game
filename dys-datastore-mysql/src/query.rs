use std::fmt::Debug;
use sqlx::Execute;

pub trait MySqlQuery: Debug + Sized {
    /// Gets a query to be run against a MySQL database.
    /// The query itself is not yet executed, and must be executed by the caller.
    fn query(&mut self) -> impl Execute<'_, sqlx::MySql>;
}