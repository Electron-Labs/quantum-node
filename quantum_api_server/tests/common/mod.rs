pub mod repository;
pub mod rocket_setup;



use dotenv::dotenv;
use lazy_static::lazy_static;
use repository::{auth_repository::insert_auth_token_random, protocol_repository::insert_electron_protocol};
use rocket::local::asynchronous::Client;
use rocket_setup::rocket_builder;
use quantum_api_server::connection::get_pool;
use tokio::sync::OnceCell; 


lazy_static!{
    static ref CLIENT: OnceCell<Client> = OnceCell::new();
}
pub async fn setup() -> &'static Client{
    CLIENT.get_or_init(|| async{
        dotenv().ok();
        println!("setting up");

        std::env::set_var("enviroment", "test");
        
        let _db_initialize = get_pool().await;

        // inserting auth token and protocol for testing
        let _ = insert_auth_token_random(get_pool().await).await;
        let _ = insert_electron_protocol(get_pool().await).await;
        Client::tracked(rocket_builder()).await.expect("Invalid rocket instance")
    }).await
}