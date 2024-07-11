mod common;
use common::{ repository::{task_repository::delete_all_task_data, user_circuit_data_repository::{delete_all_user_circuit_data, insert_random_protocol_user_circuit_data}}, setup};
use quantum_api_server::{connection::get_pool, types::{circuit_registration_status::CircuitRegistrationStatusResponse, register_circuit::RegisterCircuitResponse}};
use rocket::{http::{ContentType, Header, Status}, local::asynchronous::Client};

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";

async fn before_test(client: &Client) -> String{
    // inserting circuit data 
    let payload = include_str!("common/data/circuit/gnark.json");

    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
    .header(ContentType::JSON).body(payload).dispatch().await;

    assert_eq!(response.status(), Status::Ok);

    let res: RegisterCircuitResponse = response.into_json().await.unwrap();
    assert!(!res.circuit_hash.is_empty());

    res.circuit_hash
}


async fn after_test() {
    let _ = delete_all_user_circuit_data(get_pool().await).await;
    let _ = delete_all_task_data(get_pool().await).await;
}

#[tokio::test]
async fn test_get_circuit_reduction_status_with_invalid_circuit_hash(){
    let client = setup().await;
    let _correct_circuit_hash = before_test(client).await;

    // now fetching with incorrect hash
    let incorrect_circuit_hash = "0xa4896a3f93bf4bf58378e579f3cf193bb4af1022af7d2089f37d8bae7157b85f";

    let response = client.get(format!("/circuit/{}/status", incorrect_circuit_hash)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    
    // validating response status and content_type
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);    
    
    after_test().await; 
}

#[tokio::test]
async fn test_get_circuit_reduction_status_with_invalid_proof_type(){
    let client = setup().await;
    let circuit_hash = "0x6d42821632517e2b28b39b33aaf268a0785df7d68cccd3e01737c8de3f3ff6d7";
    let _ = insert_random_protocol_user_circuit_data(get_pool().await, circuit_hash);
    let response = client.get(format!("/circuit/{}/status", circuit_hash)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    println!("response: {:?}", response);
    // validating response status and content_type
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON); 

    after_test().await;
}

#[tokio::test]
async fn test_get_circuit_reduction_status_with_valid_circuit_hash(){
    let client = setup().await;
    let correct_circuit_hash = before_test(client).await;

    let response = client.get(format!("/circuit/{}/status", correct_circuit_hash)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    
    // validating response status and content_type
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);  

    // validating response structure
    let res: CircuitRegistrationStatusResponse = response.into_json().await.unwrap();
    assert!(!res.circuit_registration_status.is_empty());

    after_test().await;
}