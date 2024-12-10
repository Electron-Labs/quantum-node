use std::sync::Arc;

use lazy_static::lazy_static;
use once_cell::sync::Lazy;

use quantum_types::{traits::db::DB as DBTrait, types::db::mysql::{MySqlDB, SqliteDB}};
// use quantum_types::traits::db::DB as ;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, ConnectOptions, Executor, MySql, Pool, Row, SqlitePool};
use tokio::sync::OnceCell;

// pub enum DB<T> {
//     Mysql(T),
//     Sqlite(T)

// }
// lazy_static! {
//     static ref POOL: OnceCell<DB> = OnceCell::const_new();
// }

// static DB_INSTANCE: Lazy<Arc<dyn DBTrait>> = Lazy::new(|| {
//     // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     // let pool = PgPool::connect_lazy(&database_url).expect("Failed to create pool");
//     // let db = PostgresDB { pool };
//     let db = MySqlDB::initialize_pool().await;
//     // Arc::new(db)
//     let db = DB::<DBTrait>::Mysql(db);
//     db.
// });


lazy_static! {
    static ref DB_INSTANCE: Box<dyn DBTrait + Sized> = {
        // Create a PostgresDB instance (you can change this to MySQLDB)
        let db = MySqlDB::initialize_pool().await;
        Box::new(db)  // Store as Box<dyn DB>
    };
}


// static DB: dyn DBTrait;


pub async fn get_db() -> Box<dyn DBTrait> {
    // POOL.get_or_init(|| async {
        // let username = std::env::var("DB_USER").expect("DB_USER must be set.");
        // let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
        // let database = std::env::var("DB_NAME").expect("DB_NAME must be set.");

        // // let query  = sqlx::query("SELECT * from bonsai_image where image_id = ?")
        // // .bind(1.to_string());
        // // let c = SqlitePool::connect(url).await.unwrap();
        // // let temp = query.execute(&c);
        // // temp.try_get_unchecked("")

        // // let a  = ["mnbd" ];

        // let connection_options = MySqlConnectOptions::new()
        //     .username(&username)
        //     .password(&password)
        //     .database(&database)
        //     .disable_statement_logging().clone();

        // let pool_options = MySqlPoolOptions::new().min_connections(5);
        // pool_options.connect_with(connection_options).await.unwrap()
        let db = MySqlDB::initialize_pool().await;

        Box::new(db)
    // })
    // .await
}