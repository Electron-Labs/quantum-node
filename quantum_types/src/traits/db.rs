use anyhow::Result as AnyhowResult;
use sqlx::{mysql::{MySqlQueryResult, MySqlRow}, sqlite::{SqliteQueryResult, SqliteRow}, Execute, MySql, Pool, Row};
pub enum DatabaseRow {
    Mysql(MySqlRow),
    Sqlite(SqliteRow)
}

pub enum DatabaseQueryResult {
    Mysql(MySqlQueryResult),
    Sqlite(SqliteQueryResult)
}
pub trait DB : Sized + Send + Sync + 'static{
    async fn initialize_pool() -> Self;
    async fn fetch_optional(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Option<DatabaseRow>>; 
    async fn fetch_one(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<DatabaseRow>;
    async fn execute(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<DatabaseQueryResult>;
    async fn fetch_all(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Vec<DatabaseRow>>;
}