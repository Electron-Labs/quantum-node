use lazy_static::lazy_static;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use tokio::sync::Mutex;
use sqlx::MySqlPool;


lazy_static!{
    static ref POOL: Mutex<Option<Pool<MySql>>> = Mutex::new(None);
}

pub async fn get_pool() -> &'static Mutex<Option<Pool<MySql>>> {
    let mut pool = POOL.lock().await;
    match &mut *pool {
        None => {
            *pool = Some(init_pool(5).await);
        },
        _ => {}
    }
    &POOL
}

async fn init_pool(min_connections: u32) -> MySqlPool {
    let connect_options = connection_options().await;
    let pool_options = MySqlPoolOptions::new().min_connections(min_connections);
    pool_options.connect_with(connect_options).await.unwrap()
}


pub async fn connection_options() -> MySqlConnectOptions {
    let username:String;
        let password:String;
        let database: String;

        let test_or_prod = std::env::var("CARGO").expect("CARGO must be set.");
        if !test_or_prod.eq("test") {
            username = "testuser".to_string();
            password = "test".to_string();
            database = "test_quantum".to_string();
        }
        else {
            username = std::env::var("DB_USER").expect("DB_USER must be set.");
            password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
            database = std::env::var("DB_NAME").expect("DB_NAME must be set.")
        }
        
        let connect_options = MySqlConnectOptions::new()
            .username(&username)
            .password(&password)
            .database(&database);

        return connect_options;
}

pub async fn terminate_pool() {

    let test_or_prod = std::env::var("CARGO").expect("CARGO must be set.");
    if !test_or_prod.eq("test"){
        println!("cannot terminate pool in production");
    }
    let mut pool = POOL.lock().await;

    // closing pool
    pool.as_mut().unwrap().close().await;
    println!("previous pool: {:?}", pool);
    // setting pool to None
    *pool = None;
    println!("new pool: {:?}", pool);

}
