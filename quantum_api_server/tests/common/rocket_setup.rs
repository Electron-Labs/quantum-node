use quantum_types::types::config::ConfigData;
use rocket::{routes, Build, Rocket};

use quantum_api_server::routes::{ping::ping, register_circuit::register_circuit, circuit_reduction::get_circuit_reduction_status, proof::{submit_proof, get_proof_status}, auth_protocol::generate_auth_token, index::index, protocol_proof::get_protocol_proof};
pub fn rocket_builder() -> Rocket<Build> {
    
    let cors = rocket_cors::CorsOptions {
        ..Default::default()
    }.to_cors().unwrap();

    let config_data = ConfigData::new("/home/aditya/work/quantum-node/config.yaml");
    let t = rocket::Config::figment();

    rocket::custom(t).manage(config_data)
    .mount("/", routes![index, ping, register_circuit, get_circuit_reduction_status, submit_proof, get_proof_status, generate_auth_token, get_protocol_proof]).attach(cors)
}