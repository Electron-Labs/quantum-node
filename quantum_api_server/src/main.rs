use std::env;

use connection::get_pool;
use dotenv::dotenv;
use quantum_api_server::{catcher, connection, routes::{self, protocol_proof::get_protocol_proof}};
use quantum_types::types::config::ConfigData;
use quantum_utils::logger::initialize_logger;
use quantum_types;
use routes::{ping::ping, register_circuit::register_circuit, circuit_reduction::get_circuit_reduction_status, proof::{submit_proof, get_proof_status}, auth_protocol::generate_auth_token, index::index};
use catcher::{unsupported_media_type, internal_server_error};

#[macro_use] extern crate rocket;



#[launch]
async fn rocket() -> _ {
    dotenv().ok();
    env::set_var("RUST_BACKTRACE", "1");

    let cors = rocket_cors::CorsOptions {
        ..Default::default()
    }.to_cors().unwrap();

    let _guard = initialize_logger("qunatum_node_api.log");
    let config_data = ConfigData::new("./config.yaml");
    let _db_initialize = get_pool().await;

    let t = rocket::Config::figment();
    rocket::custom(t).manage(config_data).manage(_guard)
    .mount("/", routes![index, ping, register_circuit, get_circuit_reduction_status, submit_proof, get_proof_status, generate_auth_token, get_protocol_proof]).attach(cors)
    .register("/", catchers![unsupported_media_type, internal_server_error])
}