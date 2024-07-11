mod common;

use common::repository::{task_repository::{delete_all_task_data, get_task_data_count_from_circuit_hash}, user_circuit_data_repository::delete_all_user_circuit_data};
use quantum_api_server::{connection::get_pool, types::register_circuit::RegisterCircuitResponse};
use rocket::http::{ContentType, Header, Status};

use crate::common::setup; 

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";

pub async fn after_test() {
    let _ = delete_all_user_circuit_data(get_pool().await).await;
    let _ = delete_all_task_data(get_pool().await).await;
}

#[tokio::test]
async fn test_register_circuit_with_missing_payload(){
    let client = setup().await;
    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    assert_eq!(response.status(), Status::UnsupportedMediaType);   
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
async fn test_register_circuit_with_missing_data_fields(){
    let client = setup().await;
    let payload = r##"{
        "vkey": [12,131]
    }"##;

    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
    .header(ContentType::JSON).body(payload).dispatch().await;
    assert_eq!(response.status(), Status::InternalServerError);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}

#[tokio::test]
async fn test_register_circuit_should_not_register_same_circuit(){

    let client = setup().await;
    let payload = include_str!("common/data/circuit/snark.json");
    let response1 = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                       .header(ContentType::JSON).body(payload).dispatch().await;
    

    // validating status
    assert_eq!(response1.status(), Status::Ok);
    
    // validating response type
    assert_eq!(response1.content_type().unwrap(), ContentType::JSON);

    //validating response structure and fields
    let res1: RegisterCircuitResponse = response1.into_json().await.unwrap();

    let response2 = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                       .header(ContentType::JSON).body(payload).dispatch().await;

    let res2: RegisterCircuitResponse = response2.into_json().await.unwrap();
    assert!(!res2.circuit_hash.is_empty());

    // their circuit hash should be same [it is primary key]
    assert_eq!(res1.circuit_hash, res2.circuit_hash);

    // deleting circuit entry
    after_test().await;
}

#[tokio::test]
async fn test_register_circuit_with_invalid_vkey() {
    let client = setup().await;
    let payload = include_str!("common/data/invalid/circuit/invalid_vkey.json");
    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
    .header(ContentType::JSON).body(payload).dispatch().await;

    // validating status
    assert_eq!(response.status(), Status::InternalServerError);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
async fn test_register_circuit_should_return_saved_reduction_circuit(){
    let client = setup().await;
    let payload = include_str!("common/data/circuit/snark.json");

    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                        .header(ContentType::JSON).body(payload).dispatch().await;
    
    let res: RegisterCircuitResponse = response.into_json().await.unwrap();
    let circuit_hash = res.circuit_hash;

    // calling it again 
    let _ = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
    .header(ContentType::JSON).body(payload).dispatch().await;

    // fetching from task table
    let result = get_task_data_count_from_circuit_hash(get_pool().await, &circuit_hash).await.expect("Error in fetching from task table");

    // still there should be one entry
    assert_eq!(result, 1);
    
    // deleting circuit entry
    after_test().await;
}

#[tokio::test]
async fn test_register_circuit_with_invalid_proof_type(){
    let client = setup().await;
    let payload = include_str!("common/data/invalid/circuit/invalid_proof_type.json");
    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                       .header(ContentType::JSON).body(payload).dispatch().await;
    

    // validating status
    assert_eq!(response.status(), Status::InternalServerError);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}



#[tokio::test]
async fn test_register_circuit_with_valid_data_fields(){
    let client = setup().await;
    let payload = include_str!("common/data/circuit/gnark.json");
    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                       .header(ContentType::JSON).body(payload).dispatch().await;
    

    // validating status
    assert_eq!(response.status(), Status::Ok);
    
    // validating response type
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    //validating response structure and fields
    let res: RegisterCircuitResponse = response.into_json().await.unwrap();
    assert!(!res.circuit_hash.is_empty());

    // deleting circuit entry
    after_test().await;
}