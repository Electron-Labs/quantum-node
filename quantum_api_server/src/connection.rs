use std::sync::Arc;
use lazy_static::lazy_static;
use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions}, MySql, Pool};
use sqlx::MySqlPool;
use async_rwlock::RwLock;


lazy_static!{
    static ref POOL: Arc<RwLock<Option<Pool<MySql>>>> = Arc::new(RwLock::new(None));    
}

pub async fn get_pool() -> &'static Arc<RwLock<Option<Pool<MySql>>>> {

    // check if pool is already initialized
    let pool_read_guard = POOL.read().await;
    if !pool_read_guard.is_none() {
        return &POOL;
    }

    drop(pool_read_guard);
    
    
    let mut pool_write_guard = POOL.write().await;
    if pool_write_guard.is_none() { 
        *pool_write_guard = Some(init_pool(5).await);
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
    let mut pool = POOL.write().await;

    // Check if there is a pool to close
    if let Some(pool_ref) = pool.as_mut() {
        // Close the pool
        let _ = pool_ref.close().await;
        println!("previous pool closed");

        // Reset the pool to None
        *pool = None;
        println!("new pool set to None");
    } else {
        println!("No pool found to close");
    }

}
