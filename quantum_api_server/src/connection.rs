use std::sync::Arc;

use lazy_static::lazy_static;
use once_cell::sync::Lazy;

use quantum_types::{traits::db::DB, types::db::mysql::MySqlDB};
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, ConnectOptions};
// use quantum_types::traits::db::DB as ;
// use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, ConnectOptions, Executor, MySql, Pool, Row, SqlitePool};
// use tokio::sync::OnceCell;

// pub enum DB<T> {
//     Mysql(T),
//     Sqlite(T)

// }
// lazy_static! {
//     static ref POOL: OnceCell<DB> = OnceCell::const_new();
// }

static DB_INSTANCE: Lazy<Box<dyn DB + Send + Sync>> = Lazy::new(|| {
    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    // let pool = PgPool::connect_lazy(&database_url).expect("Failed to create pool");
    let pool = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async {
        // MySqlDB::initialize_pool().await
        //     .expect("Failed to create DB pool")

        let username = std::env::var("DB_USER").expect("DB_USER must be set.");
        let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
        let database = std::env::var("DB_NAME").expect("DB_NAME must be set.");

        let connection_options = MySqlConnectOptions::new()
            .username(&username)
            .password(&password)
            .database(&database)
            .disable_statement_logging().clone();

        let pool_options = MySqlPoolOptions::new().min_connections(5);
       pool_options.connect_with(connection_options).await.unwrap()
    });
    Box::new(MySqlDB{
        pool,
    })
});


// lazy_static! {
//     static ref DB_INSTANCE: Box<dyn DBTrait + Sized> = {
//         // Create a PostgresDB instance (you can change this to MySQLDB)
//         let db = MySqlDB::initialize_pool().await;
//         Box::new(db)  // Store as Box<dyn DB>
//     };
// }


// static DB: dyn DBTrait;


pub fn get_pool() -> &'static dyn DB {
        &**DB_INSTANCE
}