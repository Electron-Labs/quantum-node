mod common;
use common::{repository::{proof::delete_all_proof_data, task_repository::delete_all_task_data, user_circuit_data_repository::{delete_all_user_circuit_data, update_circuit_redn_status_user_circuit_data_completed}}, setup};
use quantum_api_server::{connection::get_pool, types::{register_circuit::RegisterCircuitResponse, submit_proof::SubmitProofResponse}};
use rocket::{http::{ContentType, Header, Status}, local::asynchronous::Client};

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";

async fn before_test(client: &Client){
    // register_circuit
    let payload = include_str!("common/data/circuit/snark.json");
    
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

#[tokio::test]
async fn test_submit_proof_with_missing_payload() {
    let client = setup().await;

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::UnsupportedMediaType);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
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


#[tokio::test]
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


#[tokio::test]
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

#[tokio::test]
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

#[tokio::test]
async fn test_submit_proof_with_repeated_proof(){
    // should return error for same prove
    let client = setup().await;

    before_test(client).await;

    let payload = include_str!("common/data/proof/snark.json");
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

#[tokio::test]
async fn test_submit_proof_with_valid_payload(){
    let client = setup().await;

    before_test(client).await;

    let payload = include_str!("common/data/proof/snark.json");
    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    
    // validating response structure type and fields
    let res: SubmitProofResponse = response.into_json().await.unwrap(); 
    assert!(!res.proof_id.is_empty());

    after_test().await;
} 