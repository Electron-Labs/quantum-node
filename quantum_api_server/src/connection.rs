use lazy_static::lazy_static;
use sqlx::{any::{AnyConnectOptions, AnyPoolOptions}, mysql::{MySqlConnectOptions, MySqlPoolOptions}, Any, ConnectOptions, MySql, Pool};
use tokio::sync::OnceCell;

lazy_static! {
    static ref POOL: OnceCell<Pool<Any>> = OnceCell::const_new();
}


pub async fn get_pool() -> &'static Pool<Any> {
    POOL.get_or_init(|| async {
        let db_url = std::env::var("DB_URL").expect("DB_URL must be set.");
        println!("{db_url}");
        let pool_options = AnyPoolOptions::new().min_connections(5);
        pool_options.connect(&db_url) .await.unwrap()
    })
    .await
}
