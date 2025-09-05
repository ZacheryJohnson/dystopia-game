
use std::fmt::Debug;

pub trait MySqlQuery: Debug + Sized {
    #[allow(clippy::doc_markdown)]
    /// Gets a query to be run against a MySQL database.
    /// The query itself is not yet executed, and must be executed by the caller.
    fn query(&mut self) -> impl sqlx::Execute<'_, sqlx::MySql>;
}