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
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
async fn test_submit_proof_with_invalid_payload(){
    let client = setup().await;
    let payload = include_str!("common/data/invalid/invalid_payload.json"); 

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                                              .header(ContentType::JSON).body(payload).dispatch().await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
async fn test_submit_proof_with_invalid_proving_scheme(){
    let client = setup().await;

    before_test(client).await;

    let invalid_proving_scheme_payload = include_str!("common/data/invalid/proof/invalid_proving_scheme.json");

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

    let invalid_proof_payload = include_str!("common/data/invalid/proof/invalid_proof.json");

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

    let invalid_pis_payload = include_str!("common/data/invalid/proof/invalid_pis.json");

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