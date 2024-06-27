use rocket::http::{ContentType, Header, Status};

mod common;
use common::setup;

const  AUTH_TOKEN: &str = "b3047d47c5d6551744680f5c3ba77de90acb84055eefdcbb";

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_invalid_path(){
    let client = setup().await;
    let invalid_url = "/invalid_path";
    let response = client.get(invalid_url).dispatch().await;
    assert_ne!(response.status(), Status::Ok);
    assert_eq!(response.status(), Status::NotFound);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}


#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_invalid_auth_token(){
    let client = setup().await;
    let invalid_auth_token = "invalid_auth_token";
    let response = client.get("/ping").header(Header::new("Authorization", invalid_auth_token)).dispatch().await;
    assert_ne!(response.status(), Status::Ok);
    assert_eq!(response.status(), Status::Unauthorized);
    assert_ne!(response.content_type().unwrap(), ContentType::JSON);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_ping(){
    let client = setup().await;
    let response = client.get("/ping").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.content_type().unwrap(), ContentType::Plain);
    assert_eq!(response.into_string().await.unwrap(), "pong");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_index(){
    let client = setup().await;
    let response = client.get("/").header(Header::new("Authorization", format!("Bearer {}", AUTH_TOKEN))).dispatch().await;
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string().await.unwrap(), "Hello, world!");
}
