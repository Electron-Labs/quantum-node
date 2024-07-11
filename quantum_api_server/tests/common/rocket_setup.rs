use quantum_types::types::config::ConfigData;
use rocket::{catchers, routes, Build, Rocket};

use quantum_api_server::{
    routes::{auth_protocol::generate_auth_token, circuit_reduction::get_circuit_reduction_status, index::index, ping::ping, proof::{get_proof_status, submit_proof}, protocol_proof::get_protocol_proof, register_circuit::register_circuit}, 
    catcher::{unsupported_media_type, internal_server_error}
};

pub fn rocket_builder() -> Rocket<Build> {
    
    let cors = rocket_cors::CorsOptions {
        ..Default::default()
    }.to_cors().unwrap();

    let config_data = ConfigData::new("../../quantum-node/config.yaml");
    let t = rocket::Config::figment();

    rocket::custom(t).manage(config_data)
    .mount("/", routes![index, ping, register_circuit, get_circuit_reduction_status, submit_proof, get_proof_status, generate_auth_token, get_protocol_proof]).attach(cors)
    .register("/", catchers![unsupported_media_type, internal_server_error])
}