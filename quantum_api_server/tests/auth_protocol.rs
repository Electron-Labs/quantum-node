mod common;
use common::{repository::protocol_repository::delete_protocol_from_protocol_name, setup};
use quantum_api_server::{connection::get_pool, types::generate_auth_token::GenerateAuthTokenResponse};
use rocket::http::{ContentType, Header, Status};

const MASTER_AUTH_TOKEN: &str = "random";

async fn after_test(protocol_name: &str){
    let _ = delete_protocol_from_protocol_name(get_pool().await, protocol_name).await;
}

#[tokio::test]
async fn test_get_auth_token_with_missing_payload(){
    let client = setup().await;
    let response = client.post("/auth/protocol").header(Header::new("Authorization", format!("Bearer {}", MASTER_AUTH_TOKEN))).dispatch().await;

    assert_eq!(response.status(), Status::UnsupportedMediaType);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}

#[tokio::test]
async fn test_get_auth_token_with_invalid_payload(){
    let client = setup().await;
    let payload = r##"{
    "random": "random"
    }"##;
    let response = client.post("/auth/protocol").header(Header::new("Authorization", format!("Bearer {}", MASTER_AUTH_TOKEN)))
                                    .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::InternalServerError);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test]
async fn test_get_auth_token_with_repeated_protocol_registration(){
    // should return error when registering twice
    let client = setup().await;    
    let payload =  r##"{
        "protocol_name": "new_protocol"
    }"##;

    let response = client.post("/auth/protocol").header(Header::new("Authorization", format!("Bearer {}", MASTER_AUTH_TOKEN)))
                                    .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res: GenerateAuthTokenResponse =response.into_json().await.unwrap();
    assert!(!res.auth_token.is_empty());

    // should return error this time

    let response = client.post("/auth/protocol").header(Header::new("Authorization", format!("Bearer {}", MASTER_AUTH_TOKEN)))
                                    .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::InternalServerError);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    after_test("new_protocol").await;
}

#[tokio::test]
async fn test_get_auth_token_with_valid_payload(){
    let client = setup().await;
    let payload = r##"{
    "protocol_name": "new_protocol"
    }"##;

    let response = client.post("/auth/protocol").header(Header::new("Authorization", format!("Bearer {}", MASTER_AUTH_TOKEN)))
                                    .header(ContentType::JSON).body(payload).dispatch().await;
    
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::JSON);

    let res: GenerateAuthTokenResponse =  response.into_json().await.unwrap();
    assert!(!res.auth_token.is_empty());

    after_test("new_protocol").await;
}