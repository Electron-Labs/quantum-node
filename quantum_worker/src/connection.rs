use lazy_static::lazy_static;
use sqlx::{ConnectOptions, mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use tokio::sync::OnceCell;

lazy_static! {
    static ref POOL: OnceCell<Pool<MySql>> = OnceCell::const_new();
}

pub async fn get_pool() -> &'static Pool<MySql> {
    POOL.get_or_init(|| async {
        let username = std::env::var("DB_USER").expect("DB_USER must be set.");
        let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
        let database = std::env::var("DB_NAME").expect("DB_NAME must be set.");

        let connection_options = MySqlConnectOptions::new()
            .username(&username)
            .password(&password)
            .database(&database)
            .disable_statement_logging().clone();

        let pool_options = MySqlPoolOptions::new().min_connections(6);
        pool_options.connect_with(connection_options).await.unwrap()
    })
    .await
}