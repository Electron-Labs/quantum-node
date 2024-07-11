mod common;
use common::{repository::{proof::{delete_all_proof_data, update_proof_status_to_verified, update_superproof_id}, task_repository::delete_all_task_data, user_circuit_data_repository::{delete_all_user_circuit_data, update_circuit_redn_status_user_circuit_data_completed}}, setup};
use quantum_api_server::{connection::get_pool, types::{proof_data::ProofDataResponse, register_circuit::RegisterCircuitResponse, submit_proof::SubmitProofResponse}};
use quantum_types::{enums::proof_status::ProofStatus, types::config::ConfigData};
use rocket::{form::validate::Contains, http::{ContentType, Header, Status}, local::asynchronous::Client};

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";
const CONFIG_DATA_PATH: &str = "../../quantum-node/config.yaml";


async fn before_test(client: &Client) -> String{
    // we have to register_circuit then update its status to complete and then submit proof and return proof_id
    let register_circuit_payload = include_str!("common/data/circuit/snark.json");

    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(register_circuit_payload).dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res: RegisterCircuitResponse = response.into_json().await.unwrap();
    assert!(!res.circuit_hash.is_empty());

    // updating proof status to complete
    let circuit_hash = res.circuit_hash;
    let _ = update_circuit_redn_status_user_circuit_data_completed(get_pool().await, &circuit_hash).await;

    // submitting correctproof

    let submit_proof_payload =  include_str!("common/data/proof/snark.json");

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                .header(ContentType::JSON).body(&submit_proof_payload).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    
    let res: SubmitProofResponse = response.into_json().await.unwrap(); 
    assert!(!res.proof_id.is_empty());
    
    res.proof_id
}

async fn after_test() {
    let _ = delete_all_user_circuit_data(get_pool().await).await;
    let _ = delete_all_task_data(get_pool().await).await;
    let _ = delete_all_proof_data(get_pool().await).await;
}

#[tokio::test]
async fn test_proof_status_with_invalid_proof_id(){
    let client = setup().await;
    let invalid_proof_id = "0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572";
    
    let response = client.get(format!("/proof/{}",invalid_proof_id)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res: ProofDataResponse = response.into_json().await.unwrap();
    let config_data = ConfigData::new(CONFIG_DATA_PATH); 
    
    assert_eq!(res.status, ProofStatus::NotFound.to_string());
    assert_eq!(res.superproof_id, -1);
    assert_eq!(res.transaction_hash, None);
    assert_eq!(res.verification_contract, config_data.verification_contract_address.to_string());
}

#[tokio::test]
async fn test_proof_status_with_valid_proof_id_invalid_superproof_id(){
    let client = setup().await;
    let proof_id = before_test(client).await;
    let invalid_superproof_id: u32 = 32;
    
    let _ = update_superproof_id(get_pool().await, &proof_id, invalid_superproof_id).await;

    let response = client.get(format!("/proof/{}", proof_id)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    assert!(response.into_string().await.contains("superproof not found in db"));

    after_test().await;
}

#[tokio::test]
async fn test_proof_status_with_valid_proof_id(){
    let client = setup().await;

    let proof_id = before_test(client).await;

    let response = client.get(format!("/proof/{}", proof_id)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res: ProofDataResponse = response.into_json().await.unwrap();
    let config_data = ConfigData::new(CONFIG_DATA_PATH);

    assert_eq!(res.status, ProofStatus::Registered.to_string());
    assert_eq!(res.verification_contract, config_data.verification_contract_address.to_string());

    after_test().await;
}