mod common;
use common::{repository::{proof::{delete_all_proof_data, update_proof_status_to_verified}, reduction_circuit_repository::{delete_all_reduction_circuit_data, insert_dummy_data_reduction_circuit}, superproof_repository::{delete_dummy_data_superproof, insert_dummy_data_superproof}, task_repository::delete_all_task_data, user_circuit_data_repository::{delete_all_user_circuit_data, update_reduction_circuit_id_user_circuit_data_completed, update_circuit_redn_status_user_circuit_data_completed}}, setup};
use quantum_api_server::{connection::get_pool, types::{register_circuit::RegisterCircuitResponse, submit_proof::SubmitProofResponse}};
use rocket::{http::{ContentType, Header, Status}, local::asynchronous::Client};

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";


async fn before_test(client: &Client) -> (String, String){
    // we have to register_circuit then update its status to complete and then submit proof and return proof_id
    let register_circuit_payload = include_str!("common/data/circuit/snark.json");

    let response = client.post("/register_circuit").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                 .header(ContentType::JSON).body(register_circuit_payload).dispatch().await;

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res1: RegisterCircuitResponse = response.into_json().await.unwrap();
    assert!(!res1.circuit_hash.is_empty());
    
    // defining reduction_circuit_hash
    let reduction_circuit_hash = "0xa4896a3f93bf4bf58378e579f3cf193bb4af1022af7d2089f37d8bae7157b85f";

    // updating proof status to complete and setting reduction_circuit_id
    let circuit_hash = res1.circuit_hash;
    let _ = update_circuit_redn_status_user_circuit_data_completed(get_pool().await, &circuit_hash).await;

    // inserting dummy data to reduction circuit
    let _ = insert_dummy_data_reduction_circuit(get_pool().await, reduction_circuit_hash).await;

    // submitting correctproof

    let submit_proof_payload =  include_str!("common/data/proof/snark.json");

    let response = client.post("/proof").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN)))
                                .header(ContentType::JSON).body(&submit_proof_payload).dispatch().await;
    
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
    
    let res2: SubmitProofResponse = response.into_json().await.unwrap(); 
    assert!(!res2.proof_id.is_empty());

    // updating proof_status to verified
    let proof_hash = res2.proof_id;
    let _ = update_proof_status_to_verified(get_pool().await, &proof_hash).await;
    
    (circuit_hash, proof_hash)
}

async fn after_test(proof_ids: &str) {
    let _ = delete_all_user_circuit_data(get_pool().await).await;
    let _ = delete_all_task_data(get_pool().await).await;
    let _ = delete_all_proof_data(get_pool().await).await;
    let _ = delete_dummy_data_superproof(get_pool().await, proof_ids).await;
    let _ = delete_all_reduction_circuit_data(get_pool().await).await;
}

#[tokio::test]
async fn test_get_protocol_proof_with_invalid_proof_id(){
    let client = setup().await;
    let invalid_proof_id = "a4896a3f93bf4bf58378e579f3cf193bb4af1022af7d2089f37d8bae7157b85f";
    let response = client.get(format!("/protocol_proof/merkle/{}", invalid_proof_id)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);
}

// #[tokio::test(flavor="multi_thread", worker_threads=4)]
// async fn test_get_protocol_proof_with_valid_proof_id_but_proof_not_verified(){
//     let client = setup().await;
//     let (circuit_hash, proof_id) = before_test(client).await;
//     let proof_ids = "[1,2,3,4,5,6,7,8,9,10]";
//     let _ = insert_dummy_data_superproof(get_pool().await, proof_ids).await;
//     let _ = update_reduction_circuit_id_user_circuit_data_completed(get_pool().await, &circuit_hash, "0xa4896a3f93bf4bf58378e579f3cf193bb4af1022af7d2089f37d8bae7157b85f").await;


//     let response = client.get(format!("/protocol_proof/merkle/{}", proof_id)).header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    
//     println!("response: {:?}", response);
//     assert_eq!(response.status(), Status::InternalServerError);
//     assert_eq!(response.content_type().unwrap(), ContentType::JSON);

//     after_test(proof_ids).await;
//     ().await;
// }