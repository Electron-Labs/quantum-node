pub mod repository;
pub mod rocket_setup;



use dotenv::dotenv;
use lazy_static::lazy_static;
use rocket::local::asynchronous::Client;
use rocket_setup::rocket_builder;
use quantum_api_server::connection::get_pool;
use tokio::sync::OnceCell; 


lazy_static!{
    static ref CLIENT: OnceCell<Client> = OnceCell::new();
}
pub async fn setup() -> &'static Client{
    dotenv().ok();
    let _db_initialize = get_pool().await;
    CLIENT.get_or_init(|| async{
        Client::tracked(rocket_builder()).await.expect("Invalid rocket instance")
    }).await
}