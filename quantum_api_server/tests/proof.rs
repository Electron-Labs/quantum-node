mod common;
use common::{db::db_connection::get_pool, repository::{proof::delete_all_proof_data, task_repository::delete_all_task_data, user_circuit_data_repository::{delete_all_user_circuit_data, update_circuit_redn_status_user_circuit_data_completed}}, setup};
use quantum_api_server::types::{register_circuit::RegisterCircuitResponse, submit_proof::SubmitProofResponse};
use quantum_types::types::halo2_plonk::Halo2PlonkPis;
use rocket::{http::{ContentType, Header, Status}, local::asynchronous::Client, response};

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";

async fn before_test(client: &Client){
    // register_circuit
    let payload = r##"{
        "vkey": [7,0,0,0,103,114,111,116,104,49,54,5,0,0,0,98,110,49,50,56,2,0,0,0,3,0,0,0,76,0,0,0,54,49,56,55,55,55,51,55,56,54,48,57,57,50,56,51,53,51,51,52,49,55,55,57,48,51,55,55,57,57,57,49,50,53,48,56,52,55,54,56,55,52,54,49,56,54,51,55,57,51,53,55,55,50,57,49,52,53,49,54,49,56,57,48,50,51,51,50,49,49,54,56,48,55,53,49,77,0,0,0,49,48,48,56,49,49,50,52,53,49,56,55,50,54,53,57,53,50,52,48,55,53,57,54,49,57,50,49,57,56,53,52,52,54,53,56,53,50,54,49,57,55,52,52,48,53,49,50,52,49,54,53,56,55,53,49,55,56,49,51,56,48,57,56,49,50,55,57,55,56,54,56,54,54,55,57,48,1,0,0,0,49,3,0,0,0,2,0,0,0,77,0,0,0,49,49,55,51,53,57,50,50,51,51,50,54,48,57,51,55,57,52,51,52,51,55,50,56,48,50,56,52,48,50,55,53,55,57,51,53,50,55,50,53,52,49,55,52,51,53,49,54,50,51,52,55,53,49,48,57,51,57,54,55,56,48,50,53,57,57,51,55,51,57,52,50,55,49,50,49,50,77,0,0,0,50,49,55,57,51,54,48,54,50,56,49,52,56,50,49,48,51,57,51,54,51,52,49,52,53,55,49,55,53,56,50,57,53,49,55,57,53,53,49,50,55,57,50,52,51,52,54,57,50,50,49,53,48,54,50,49,57,52,50,50,53,57,50,57,54,55,54,55,52,53,52,50,49,52,49,51,50,2,0,0,0,76,0,0,0,52,50,50,54,49,51,50,48,48,53,51,49,51,51,50,57,50,56,55,53,55,55,56,56,51,52,55,48,57,55,52,56,53,53,54,50,50,57,55,57,53,55,56,56,54,56,52,55,49,56,51,48,54,57,57,51,51,53,51,49,51,57,49,54,56,54,53,49,53,54,49,55,51,53,53,52,77,0,0,0,49,52,57,54,55,49,54,52,55,52,53,51,56,52,49,50,56,51,57,51,51,50,49,49,53,50,49,56,55,57,54,49,53,48,54,55,48,54,56,51,49,52,50,54,56,55,55,57,54,50,57,51,48,50,57,48,51,51,54,53,51,57,50,55,54,56,57,57,54,51,49,52,52,56,53,53,57,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,2,0,0,0,77,0,0,0,49,48,56,53,55,48,52,54,57,57,57,48,50,51,48,53,55,49,51,53,57,52,52,53,55,48,55,54,50,50,51,50,56,50,57,52,56,49,51,55,48,55,53,54,51,53,57,53,55,56,53,49,56,48,56,54,57,57,48,53,49,57,57,57,51,50,56,53,54,53,53,56,53,50,55,56,49,77,0,0,0,49,49,53,53,57,55,51,50,48,51,50,57,56,54,51,56,55,49,48,55,57,57,49,48,48,52,48,50,49,51,57,50,50,56,53,55,56,51,57,50,53,56,49,50,56,54,49,56,50,49,49,57,50,53,51,48,57,49,55,52,48,51,49,53,49,52,53,50,51,57,49,56,48,53,54,51,52,2,0,0,0,76,0,0,0,56,52,57,53,54,53,51,57,50,51,49,50,51,52,51,49,52,49,55,54,48,52,57,55,51,50,52,55,52,56,57,50,55,50,52,51,56,52,49,56,49,57,48,53,56,55,50,54,51,54,48,48,49,52,56,55,55,48,50,56,48,54,52,57,51,48,54,57,53,56,49,48,49,57,51,48,76,0,0,0,52,48,56,50,51,54,55,56,55,53,56,54,51,52,51,51,54,56,49,51,51,50,50,48,51,52,48,51,49,52,53,52,51,53,53,54,56,51,49,54,56,53,49,51,50,55,53,57,51,52,48,49,50,48,56,49,48,53,55,52,49,48,55,54,50,49,52,49,50,48,48,57,51,53,51,49,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,2,0,0,0,76,0,0,0,55,48,54,55,50,55,54,55,55,53,52,54,52,52,52,49,53,49,48,57,50,49,52,49,48,48,57,51,48,54,48,52,52,52,56,54,53,54,54,50,54,52,48,50,48,54,55,53,54,50,51,51,51,52,51,49,57,55,50,50,52,52,53,52,49,48,57,51,56,53,54,48,50,52,51,55,76,0,0,0,54,54,49,53,49,51,54,53,57,54,54,49,48,56,49,50,51,54,55,48,48,53,50,53,48,52,56,52,54,51,53,50,57,54,51,49,56,53,52,48,54,54,57,56,53,51,49,55,50,57,52,56,55,54,52,57,56,56,56,55,57,52,49,53,48,55,56,50,56,57,51,52,49,56,49,49,2,0,0,0,76,0,0,0,56,48,50,53,48,52,57,48,53,56,51,50,53,53,52,49,51,51,48,52,56,55,56,48,54,48,54,49,54,53,51,48,57,53,49,56,48,56,56,56,54,51,51,53,48,57,55,49,52,57,53,55,50,57,51,49,53,54,56,50,55,50,52,51,56,57,53,54,51,56,54,54,54,51,48,53,77,0,0,0,50,49,48,54,57,50,56,49,50,48,50,49,54,52,55,54,53,51,51,50,55,48,48,51,57,49,53,51,56,54,53,49,48,56,57,52,49,56,51,55,55,54,55,54,53,57,54,55,51,49,54,48,48,57,51,49,56,57,50,51,54,48,56,54,56,51,56,49,56,56,49,57,50,51,52,48,53,2,0,0,0,1,0,0,0,49,1,0,0,0,48,2,0,0,0,3,0,0,0,2,0,0,0,76,0,0,0,54,56,49,49,52,53,50,51,55,56,51,49,53,56,55,49,51,51,49,57,55,55,57,57,51,50,50,49,52,50,53,57,50,55,53,53,48,50,50,53,57,53,52,53,52,56,49,55,48,53,49,55,52,56,52,56,55,51,51,52,56,50,53,56,50,48,55,48,55,50,55,56,48,52,54,50,76,0,0,0,50,55,56,51,49,55,55,51,48,52,57,54,52,53,51,56,57,56,54,48,53,57,51,50,56,49,49,50,53,55,56,50,49,55,53,49,49,53,55,50,51,55,51,53,54,48,52,55,56,52,49,56,53,56,56,52,55,48,51,55,48,50,50,52,54,49,52,53,51,49,51,51,51,53,52,57,2,0,0,0,75,0,0,0,53,54,57,51,57,56,57,57,50,54,53,49,50,54,51,57,53,48,50,48,50,49,53,51,54,50,57,52,48,53,48,48,56,53,57,57,50,49,48,52,57,48,49,48,56,50,49,56,55,50,53,56,52,49,56,48,56,53,48,52,53,54,56,57,54,55,48,53,55,51,56,57,52,57,53,77,0,0,0,50,49,50,55,48,51,56,48,52,56,54,56,56,56,49,49,53,50,48,54,50,52,54,54,53,56,53,48,50,53,48,53,49,57,55,54,50,53,55,56,51,57,53,53,54,48,49,52,50,53,53,56,57,57,56,54,54,48,54,49,56,50,48,57,52,53,54,49,50,50,48,55,56,51,55,55,49,2,0,0,0,77,0,0,0,49,56,51,53,49,57,55,57,51,51,51,55,53,54,56,51,56,48,49,57,48,49,53,52,55,57,56,51,57,54,54,50,52,49,55,51,56,53,56,55,57,53,55,50,48,54,49,56,56,54,50,48,49,49,55,56,57,57,52,51,56,49,56,52,48,54,51,54,57,54,53,49,55,53,48,55,56,77,0,0,0,49,49,55,54,48,56,57,50,53,54,54,54,56,53,57,51,53,56,54,57,57,50,50,50,57,55,53,48,51,55,49,49,50,53,53,54,51,54,48,50,55,57,52,50,56,49,53,54,51,56,55,54,57,50,49,49,55,52,50,53,54,57,55,51,53,57,56,51,51,51,56,52,51,54,49,56,56,3,0,0,0,2,0,0,0,77,0,0,0,49,52,53,50,56,49,57,52,53,51,57,56,57,52,48,52,57,49,54,48,49,53,51,57,51,51,51,51,56,54,55,55,54,55,57,57,55,51,53,55,48,53,56,53,56,54,48,54,57,57,50,57,50,52,57,54,48,54,50,55,56,50,48,54,50,54,54,48,50,57,53,49,51,48,49,49,52,77,0,0,0,49,50,48,51,51,51,55,54,51,52,48,57,51,48,48,57,57,49,50,51,54,48,50,52,55,54,56,49,56,56,50,53,50,56,48,52,56,48,55,49,54,51,53,50,48,53,51,49,48,57,48,48,52,56,55,48,54,48,54,54,48,52,52,48,57,51,51,51,51,55,55,57,55,50,55,51,56,2,0,0,0,77,0,0,0,49,53,53,50,56,57,48,55,51,48,53,48,54,57,54,53,57,52,53,52,53,49,48,51,51,51,49,49,57,54,53,52,51,53,48,54,48,53,48,53,56,48,53,54,55,51,50,52,52,49,52,53,51,55,51,53,51,54,57,54,52,55,57,52,49,54,56,56,55,50,53,51,56,56,51,54,53,77,0,0,0,49,50,56,54,55,56,57,48,50,50,54,51,54,49,54,56,51,50,57,51,54,52,57,48,49,54,53,51,52,48,55,57,50,57,53,55,50,57,49,55,49,53,57,50,54,52,56,55,48,48,50,57,50,49,48,53,49,48,57,57,49,53,57,53,51,53,50,51,56,53,49,55,48,51,49,50,57,2,0,0,0,77,0,0,0,49,54,48,52,55,54,56,48,50,57,54,52,54,56,50,49,57,55,49,57,48,57,51,49,50,57,50,51,48,53,51,51,53,50,53,57,50,48,50,56,52,51,53,52,56,55,56,52,48,57,49,52,55,53,48,55,52,57,51,53,50,55,48,54,49,55,56,48,52,50,55,56,56,53,54,51,54,76,0,0,0,55,55,48,55,52,48,57,49,51,49,48,52,48,51,52,57,54,50,48,56,50,56,53,55,56,57,50,56,52,49,54,49,51,53,56,56,55,54,51,54,50,52,55,49,57,51,55,48,53,49,49,52,56,52,57,48,54,57,51,55,48,49,57,55,48,57,56,52,54,57,51,55,57,55,52,57,3,0,0,0,3,0,0,0,76,0,0,0,55,52,50,53,49,50,56,55,57,51,55,48,55,52,48,57,55,57,53,56,55,48,48,52,52,48,50,49,48,56,56,54,51,54,52,54,50,56,52,57,54,57,55,55,50,50,51,48,57,50,53,50,49,54,57,56,51,53,51,49,54,49,52,48,57,52,49,52,50,53,56,52,49,50,54,53,76,0,0,0,50,48,57,48,55,57,54,54,49,50,53,53,50,57,57,51,55,53,50,52,54,50,54,57,56,50,51,55,57,53,48,53,50,53,49,49,54,49,57,48,53,52,56,53,53,48,57,49,54,56,49,52,53,49,52,52,54,57,50,53,49,54,53,56,49,53,57,49,51,49,57,51,57,54,56,52,1,0,0,0,49,3,0,0,0,77,0,0,0,49,53,51,55,52,49,55,51,55,56,55,49,54,49,51,54,51,50,56,56,48,55,51,48,50,57,56,53,56,52,56,48,53,49,48,52,55,48,57,51,53,49,56,54,53,49,54,54,51,48,50,48,52,56,57,50,55,52,50,50,49,52,56,54,57,57,55,57,53,55,48,51,52,57,48,57,51,76,0,0,0,57,50,50,53,52,53,50,49,53,50,57,57,55,54,56,50,52,49,52,55,49,57,57,57,49,52,48,50,57,49,54,49,54,50,48,51,56,51,49,56,48,57,55,50,51,55,53,57,57,49,49,48,55,52,52,49,48,56,54,48,51,49,57,54,51,50,49,49,52,54,48,54,50,52,57,55,1,0,0,0,49,3,0,0,0,76,0,0,0,53,52,53,50,53,50,57,48,53,48,50,53,57,55,55,55,54,57,50,55,51,50,49,50,54,51,54,52,52,48,51,49,56,49,56,52,52,48,48,53,51,51,51,53,55,54,48,48,51,51,48,49,48,56,49,52,54,54,56,50,52,54,54,55,53,53,55,57,50,49,54,51,55,56,56,54,76,0,0,0,55,52,55,54,48,49,49,56,51,48,52,55,52,54,54,49,50,53,54,50,57,54,49,56,54,48,49,56,49,50,55,48,51,57,54,51,55,49,49,55,48,54,48,52,49,53,49,56,54,57,52,55,49,53,52,52,50,55,49,57,57,53,50,57,57,56,50,54,52,52,53,57,50,51,55,49,1,0,0,0,49],
        "num_public_inputs": 2,
        "proof_type": "Groth16"
        }"##;
    
    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                        .header(ContentType::JSON).body(payload).dispatch().await;
    
    let res: RegisterCircuitResponse = response.into_json().await.unwrap();
    assert!(!res.circuit_hash.is_empty());

    // update reduction status to completed
    let circuit_hash = res.circuit_hash;
    let _ = update_circuit_redn_status_user_circuit_data_completed(get_pool().await, &circuit_hash).await;
}

async fn after_test() {
    let _ = delete_all_task_data(get_pool().await).await;
    let _ = delete_all_user_circuit_data(get_pool().await).await;
    let _ = delete_all_proof_data(get_pool().await).await;
}

#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_missing_payload() {
    let client = setup().await;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::UnsupportedMediaType);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_invalid_payload(){
    let client = setup().await;
    let payload = r##"{
    "circuit_hash": "0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572",
    "proof_type":"GnarkGroth16"
    }"##; 

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                                              .header(ContentType::JSON).body(payload).dispatch().await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_invalid_proving_scheme(){
    let client = setup().await;

    before_test(client).await;

    let invalid_proving_scheme_payload = r##"{
    "proof": [3,0,0,0,77,0,0,0,49,49,48,54,49,56,51,53,52,48,57,51,52,56,52,53,49,48,53,50,50,53,55,54,55,48,50,50,57,56,55,56,57,55,54,50,50,48,53,52,57,48,54,50,48,56,55,48,51,52,57,53,49,51,51,49,55,53,52,54,55,53,57,54,52,53,54,51,48,52,54,52,54,52,51,51,52,77,0,0,0,49,53,49,53,50,49,55,51,52,48,55,52,48,49,55,48,48,48,50,56,55,53,52,52,56,57,50,49,50,56,52,52,56,48,53,50,56,53,53,48,54,51,51,50,57,52,57,55,52,52,49,51,48,56,56,52,53,56,54,57,52,51,51,57,50,50,56,53,52,50,55,57,49,53,55,50,55,1,0,0,0,49,3,0,0,0,2,0,0,0,76,0,0,0,57,54,53,49,52,48,56,51,56,51,54,51,56,48,54,50,49,57,52,56,51,56,57,49,57,50,56,48,52,57,55,53,55,49,57,52,54,51,51,56,53,48,56,52,54,55,56,54,48,51,49,52,56,48,48,56,52,49,49,56,52,52,57,53,57,57,53,52,52,51,48,52,54,57,48,56,76,0,0,0,52,51,49,50,52,57,55,53,53,55,55,54,56,49,49,57,49,57,53,54,48,56,56,48,50,50,55,55,53,57,52,52,49,56,56,50,51,55,55,51,48,50,50,55,52,49,55,53,48,52,56,48,53,52,55,56,48,50,55,53,54,57,48,49,52,56,48,56,49,52,51,51,50,48,48,53,2,0,0,0,76,0,0,0,51,56,53,52,54,56,54,52,52,49,51,50,48,50,55,51,51,50,53,48,52,48,55,56,55,55,53,49,48,57,50,53,54,55,55,56,50,50,52,57,51,53,54,50,49,48,51,53,56,49,50,49,51,52,55,56,48,50,56,55,55,57,56,49,55,56,57,56,52,56,55,57,51,56,55,49,76,0,0,0,50,48,56,53,56,52,54,53,50,52,50,51,49,51,57,52,49,55,51,56,50,49,50,54,51,48,53,52,48,55,52,56,48,54,56,49,57,54,52,49,50,53,50,52,49,51,50,56,48,50,52,53,51,54,51,57,51,51,48,57,56,54,50,56,49,48,56,51,56,55,52,53,57,51,57,48,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,76,0,0,0,53,50,51,56,52,48,57,52,54,56,49,49,49,57,53,53,57,57,52,52,52,48,51,51,57,54,50,52,54,55,56,50,52,53,53,57,49,52,50,54,49,48,56,53,54,55,53,52,50,56,48,57,56,50,50,55,53,57,53,49,56,51,57,56,52,55,50,48,48,53,54,55,57,56,50,52,76,0,0,0,53,56,53,51,54,54,51,56,55,57,55,50,53,48,50,54,51,57,56,54,55,53,57,54,50,50,48,55,55,55,57,49,55,54,53,52,56,57,50,56,51,53,56,52,56,49,52,53,57,56,54,57,55,55,51,49,50,53,54,48,50,56,53,54,56,57,56,51,51,55,49,50,55,49,53,48,1,0,0,0,49,7,0,0,0,103,114,111,116,104,49,54,5,0,0,0,98,110,49,50,56],
    "pis":[2,0,0,0,4,0,0,0,51,53,56,52,2,0,0,0,53,54],
    "circuit_hash": "0x620d0638da45af1bbdf1f60d264382ebe21cd6a20de152479c5d3cd0a6a376de",
    "proof_type":"Plonky2"
    }"##;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(invalid_proving_scheme_payload).dispatch().await;
    
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    after_test().await;
}


#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_invalid_proof(){
    let client = setup().await;

    before_test(client).await;

    let invalid_proof_payload = r##"{
    "proof": [3,0,0,0,77,0,0,0,49,49,48,54,49,56,51,53],
    "pis":[2,0,0,0,4,0,0,0,51,53,56,52,2,0,0,0,53,54],
    "circuit_hash": "0x620d0638da45af1bbdf1f60d264382ebe21cd6a20de152479c5d3cd0a6a376de",
    "proof_type":"Groth16"
    }"##;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(invalid_proof_payload).dispatch().await;
    
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    after_test().await;
}

#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_invalid_pis(){
    let client = setup().await;

    before_test(client).await;

    let invalid_pis_payload = r##"{
    "proof": [3,0,0,0,77,0,0,0,49,49,48,54,49,56,51,53,52,48,57,51,52,56,52,53,49,48,53,50,50,53,55,54,55,48,50,50,57,56,55,56,57,55,54,50,50,48,53,52,57,48,54,50,48,56,55,48,51,52,57,53,49,51,51,49,55,53,52,54,55,53,57,54,52,53,54,51,48,52,54,52,54,52,51,51,52,77,0,0,0,49,53,49,53,50,49,55,51,52,48,55,52,48,49,55,48,48,48,50,56,55,53,52,52,56,57,50,49,50,56,52,52,56,48,53,50,56,53,53,48,54,51,51,50,57,52,57,55,52,52,49,51,48,56,56,52,53,56,54,57,52,51,51,57,50,50,56,53,52,50,55,57,49,53,55,50,55,1,0,0,0,49,3,0,0,0,2,0,0,0,76,0,0,0,57,54,53,49,52,48,56,51,56,51,54,51,56,48,54,50,49,57,52,56,51,56,57,49,57,50,56,48,52,57,55,53,55,49,57,52,54,51,51,56,53,48,56,52,54,55,56,54,48,51,49,52,56,48,48,56,52,49,49,56,52,52,57,53,57,57,53,52,52,51,48,52,54,57,48,56,76,0,0,0,52,51,49,50,52,57,55,53,53,55,55,54,56,49,49,57,49,57,53,54,48,56,56,48,50,50,55,55,53,57,52,52,49,56,56,50,51,55,55,51,48,50,50,55,52,49,55,53,48,52,56,48,53,52,55,56,48,50,55,53,54,57,48,49,52,56,48,56,49,52,51,51,50,48,48,53,2,0,0,0,76,0,0,0,51,56,53,52,54,56,54,52,52,49,51,50,48,50,55,51,51,50,53,48,52,48,55,56,55,55,53,49,48,57,50,53,54,55,55,56,50,50,52,57,51,53,54,50,49,48,51,53,56,49,50,49,51,52,55,56,48,50,56,55,55,57,56,49,55,56,57,56,52,56,55,57,51,56,55,49,76,0,0,0,50,48,56,53,56,52,54,53,50,52,50,51,49,51,57,52,49,55,51,56,50,49,50,54,51,48,53,52,48,55,52,56,48,54,56,49,57,54,52,49,50,53,50,52,49,51,50,56,48,50,52,53,51,54,51,57,51,51,48,57,56,54,50,56,49,48,56,51,56,55,52,53,57,51,57,48,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,76,0,0,0,53,50,51,56,52,48,57,52,54,56,49,49,49,57,53,53,57,57,52,52,52,48,51,51,57,54,50,52,54,55,56,50,52,53,53,57,49,52,50,54,49,48,56,53,54,55,53,52,50,56,48,57,56,50,50,55,53,57,53,49,56,51,57,56,52,55,50,48,48,53,54,55,57,56,50,52,76,0,0,0,53,56,53,51,54,54,51,56,55,57,55,50,53,48,50,54,51,57,56,54,55,53,57,54,50,50,48,55,55,55,57,49,55,54,53,52,56,57,50,56,51,53,56,52,56,49,52,53,57,56,54,57,55,55,51,49,50,53,54,48,50,56,53,54,56,57,56,51,51,55,49,50,55,49,53,48,1,0,0,0,49,7,0,0,0,103,114,111,116,104,49,54,5,0,0,0,98,110,49,50,56],
    "pis":[2,0,0,0,4,0,0,0],
    "circuit_hash": "0x620d0638da45af1bbdf1f60d264382ebe21cd6a20de152479c5d3cd0a6a376de",
    "proof_type":"Groth16"
    }"##;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(invalid_pis_payload).dispatch().await;
    
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    after_test().await;
}

#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_repeated_proof(){
    // should return error for same prove
    let client = setup().await;

    before_test(client).await;

    let payload = r##"{
    "proof": [3,0,0,0,77,0,0,0,49,49,48,54,49,56,51,53,52,48,57,51,52,56,52,53,49,48,53,50,50,53,55,54,55,48,50,50,57,56,55,56,57,55,54,50,50,48,53,52,57,48,54,50,48,56,55,48,51,52,57,53,49,51,51,49,55,53,52,54,55,53,57,54,52,53,54,51,48,52,54,52,54,52,51,51,52,77,0,0,0,49,53,49,53,50,49,55,51,52,48,55,52,48,49,55,48,48,48,50,56,55,53,52,52,56,57,50,49,50,56,52,52,56,48,53,50,56,53,53,48,54,51,51,50,57,52,57,55,52,52,49,51,48,56,56,52,53,56,54,57,52,51,51,57,50,50,56,53,52,50,55,57,49,53,55,50,55,1,0,0,0,49,3,0,0,0,2,0,0,0,76,0,0,0,57,54,53,49,52,48,56,51,56,51,54,51,56,48,54,50,49,57,52,56,51,56,57,49,57,50,56,48,52,57,55,53,55,49,57,52,54,51,51,56,53,48,56,52,54,55,56,54,48,51,49,52,56,48,48,56,52,49,49,56,52,52,57,53,57,57,53,52,52,51,48,52,54,57,48,56,76,0,0,0,52,51,49,50,52,57,55,53,53,55,55,54,56,49,49,57,49,57,53,54,48,56,56,48,50,50,55,55,53,57,52,52,49,56,56,50,51,55,55,51,48,50,50,55,52,49,55,53,48,52,56,48,53,52,55,56,48,50,55,53,54,57,48,49,52,56,48,56,49,52,51,51,50,48,48,53,2,0,0,0,76,0,0,0,51,56,53,52,54,56,54,52,52,49,51,50,48,50,55,51,51,50,53,48,52,48,55,56,55,55,53,49,48,57,50,53,54,55,55,56,50,50,52,57,51,53,54,50,49,48,51,53,56,49,50,49,51,52,55,56,48,50,56,55,55,57,56,49,55,56,57,56,52,56,55,57,51,56,55,49,76,0,0,0,50,48,56,53,56,52,54,53,50,52,50,51,49,51,57,52,49,55,51,56,50,49,50,54,51,48,53,52,48,55,52,56,48,54,56,49,57,54,52,49,50,53,50,52,49,51,50,56,48,50,52,53,51,54,51,57,51,51,48,57,56,54,50,56,49,48,56,51,56,55,52,53,57,51,57,48,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,76,0,0,0,53,50,51,56,52,48,57,52,54,56,49,49,49,57,53,53,57,57,52,52,52,48,51,51,57,54,50,52,54,55,56,50,52,53,53,57,49,52,50,54,49,48,56,53,54,55,53,52,50,56,48,57,56,50,50,55,53,57,53,49,56,51,57,56,52,55,50,48,48,53,54,55,57,56,50,52,76,0,0,0,53,56,53,51,54,54,51,56,55,57,55,50,53,48,50,54,51,57,56,54,55,53,57,54,50,50,48,55,55,55,57,49,55,54,53,52,56,57,50,56,51,53,56,52,56,49,52,53,57,56,54,57,55,55,51,49,50,53,54,48,50,56,53,54,56,57,56,51,51,55,49,50,55,49,53,48,1,0,0,0,49,7,0,0,0,103,114,111,116,104,49,54,5,0,0,0,98,110,49,50,56],
    "pis":[2,0,0,0,4,0,0,0,51,53,56,52,2,0,0,0,53,54],
    "circuit_hash": "0x620d0638da45af1bbdf1f60d264382ebe21cd6a20de152479c5d3cd0a6a376de",
    "proof_type":"Groth16"
    }"##;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                .header(ContentType::JSON).body(payload).dispatch().await;
    
    // it should work correctly for the first time

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    
    // validating response structure type and fields
    let res: SubmitProofResponse = response.into_json().await.unwrap(); 
    assert!(!res.proof_id.is_empty());

    // should be giving error for the second time

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(payload).dispatch().await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    after_test().await;

}

#[tokio::test(flavor="multi_thread", worker_threads = 1)]
async fn test_submit_proof_with_valid_payload(){
    let client = setup().await;

    before_test(client).await;

    let payload = r##"{
    "proof": [3,0,0,0,77,0,0,0,49,49,48,54,49,56,51,53,52,48,57,51,52,56,52,53,49,48,53,50,50,53,55,54,55,48,50,50,57,56,55,56,57,55,54,50,50,48,53,52,57,48,54,50,48,56,55,48,51,52,57,53,49,51,51,49,55,53,52,54,55,53,57,54,52,53,54,51,48,52,54,52,54,52,51,51,52,77,0,0,0,49,53,49,53,50,49,55,51,52,48,55,52,48,49,55,48,48,48,50,56,55,53,52,52,56,57,50,49,50,56,52,52,56,48,53,50,56,53,53,48,54,51,51,50,57,52,57,55,52,52,49,51,48,56,56,52,53,56,54,57,52,51,51,57,50,50,56,53,52,50,55,57,49,53,55,50,55,1,0,0,0,49,3,0,0,0,2,0,0,0,76,0,0,0,57,54,53,49,52,48,56,51,56,51,54,51,56,48,54,50,49,57,52,56,51,56,57,49,57,50,56,48,52,57,55,53,55,49,57,52,54,51,51,56,53,48,56,52,54,55,56,54,48,51,49,52,56,48,48,56,52,49,49,56,52,52,57,53,57,57,53,52,52,51,48,52,54,57,48,56,76,0,0,0,52,51,49,50,52,57,55,53,53,55,55,54,56,49,49,57,49,57,53,54,48,56,56,48,50,50,55,55,53,57,52,52,49,56,56,50,51,55,55,51,48,50,50,55,52,49,55,53,48,52,56,48,53,52,55,56,48,50,55,53,54,57,48,49,52,56,48,56,49,52,51,51,50,48,48,53,2,0,0,0,76,0,0,0,51,56,53,52,54,56,54,52,52,49,51,50,48,50,55,51,51,50,53,48,52,48,55,56,55,55,53,49,48,57,50,53,54,55,55,56,50,50,52,57,51,53,54,50,49,48,51,53,56,49,50,49,51,52,55,56,48,50,56,55,55,57,56,49,55,56,57,56,52,56,55,57,51,56,55,49,76,0,0,0,50,48,56,53,56,52,54,53,50,52,50,51,49,51,57,52,49,55,51,56,50,49,50,54,51,48,53,52,48,55,52,56,48,54,56,49,57,54,52,49,50,53,50,52,49,51,50,56,48,50,52,53,51,54,51,57,51,51,48,57,56,54,50,56,49,48,56,51,56,55,52,53,57,51,57,48,2,0,0,0,1,0,0,0,49,1,0,0,0,48,3,0,0,0,76,0,0,0,53,50,51,56,52,48,57,52,54,56,49,49,49,57,53,53,57,57,52,52,52,48,51,51,57,54,50,52,54,55,56,50,52,53,53,57,49,52,50,54,49,48,56,53,54,55,53,52,50,56,48,57,56,50,50,55,53,57,53,49,56,51,57,56,52,55,50,48,48,53,54,55,57,56,50,52,76,0,0,0,53,56,53,51,54,54,51,56,55,57,55,50,53,48,50,54,51,57,56,54,55,53,57,54,50,50,48,55,55,55,57,49,55,54,53,52,56,57,50,56,51,53,56,52,56,49,52,53,57,56,54,57,55,55,51,49,50,53,54,48,50,56,53,54,56,57,56,51,51,55,49,50,55,49,53,48,1,0,0,0,49,7,0,0,0,103,114,111,116,104,49,54,5,0,0,0,98,110,49,50,56],
    "pis":[2,0,0,0,4,0,0,0,51,53,56,52,2,0,0,0,53,54],
    "circuit_hash": "0x620d0638da45af1bbdf1f60d264382ebe21cd6a20de152479c5d3cd0a6a376de",
    "proof_type":"Groth16"
    }"##;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    
    // validating response structure type and fields
    let res: SubmitProofResponse = response.into_json().await.unwrap(); 
    assert!(!res.proof_id.is_empty());

    after_test().await;
} 