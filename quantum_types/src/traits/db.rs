use sqlx::{mysql::{MySqlQueryResult, MySqlRow}, sqlite::{SqliteQueryResult, SqliteRow}};
pub enum DatabaseRow {
    Mysql(MySqlRow),
    Sqlite(SqliteRow)
}

pub enum DatabaseQueryResult {
    Mysql(MySqlQueryResult),
    Sqlite(SqliteQueryResult)
}

// #[async_trait]
pub trait DB : Send + Sync {
    // async fn initialize_pool() -> Self;
    // async fn fetch_optional(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Option<MySqlRow>>; 
    // async fn fetch_one(&self, query: &str, arguments: Vec<&str>) -> Result<MySqlRow, sqlx::Error>;
    // async fn execute(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<MySqlQueryResult>;
    // async fn fetch_all(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Vec<MySqlRow>>;
}