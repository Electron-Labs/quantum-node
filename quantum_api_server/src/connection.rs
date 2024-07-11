use lazy_static::lazy_static;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use tokio::sync::OnceCell;

lazy_static! {
    static ref POOL: OnceCell<Pool<MySql>> = OnceCell::const_new();
}

pub async fn get_pool() -> &'static Pool<MySql> {
    POOL.get_or_init(|| async {
        let username:String;
        let password:String;
        let database: String;

        let env = std::env::var("enviroment");
        if env.is_ok() && env.unwrap() == "test" {
            println!("Running tests: using test database");
            username = "testuser".to_string();
            password = "test".to_string();
            database = "test_quantum".to_string();
        }
        else {
            username = std::env::var("DB_USER").expect("DB_USER must be set.");
            password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
            database = std::env::var("DB_NAME").expect("DB_NAME must be set.")
        }
        
        let connection_options = MySqlConnectOptions::new()
            .username(&username)
            .password(&password)
            .database(&database);


        let pool_options = MySqlPoolOptions::new().min_connections(5);
        pool_options.connect_with(connection_options).await.unwrap()
    })
    .await
}