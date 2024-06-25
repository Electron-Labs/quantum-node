use lazy_static::lazy_static;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use tokio::sync::OnceCell;

lazy_static! {
    static ref POOL: OnceCell<Pool<MySql>> = OnceCell::const_new();
}

pub async fn get_pool() -> &'static Pool<MySql> {
    POOL.get_or_init(|| async {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE URL must be set.");
        let pool_options = MySqlPoolOptions::new().min_connections(5);
        pool_options.connect(&database_url).await.unwrap()
    })
    .await
}