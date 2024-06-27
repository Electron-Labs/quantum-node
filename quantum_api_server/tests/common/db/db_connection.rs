use lazy_static::lazy_static;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use tokio::sync::OnceCell;

lazy_static! {
    static ref POOL: OnceCell<Pool<MySql>> = OnceCell::const_new();
}

pub async fn get_pool() -> &'static Pool<MySql> {
    POOL.get_or_init(|| async {
        let username = "testuser";
        let password = "test";
        let database = "quantum"; // TODO change it to test_quantum;

        let connection_options = MySqlConnectOptions::new()
            .username(username)
            .password(password)
            .database(database);

        let pool_options = MySqlPoolOptions::new().min_connections(5);
        pool_options.connect_with(connection_options).await.unwrap()
    })
    .await
}