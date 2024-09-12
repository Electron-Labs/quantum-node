use std::time::Duration;

use bonsai_sdk::non_blocking::{Client, SessionId};
use quantum_db::repository::{proof_repository::update_session_id_in_proof, superproof_repository::update_session_id_superproof};
use risc0_zkvm::Receipt;

use crate::connection::get_pool;

use anyhow::{anyhow, Ok, Result as AnyhowResult};
pub async fn execute_proof_reduction(input_data: Vec<u8>, image_id: &str, proof_id: u64) -> AnyhowResult<Option<Receipt>> {
    
    let client = Client::from_env(risc0_zkvm::VERSION)?;

    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let assumptions: Vec<String> = vec![];

    // Wether to run in execute only mode
    let execute_only = false;

    //TODO: store in DB
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    println!("sessionId: {:?}", session.uuid);

    update_session_id_in_proof(get_pool().await, proof_id, &session.uuid);

    let receipt = check_session_status(session, client).await?;
    Ok(receipt)
}

pub async fn execute_aggregation(input_data: Vec<u8>, image_id: &str, assumptions: Vec<String>, superproof_id: u64) -> AnyhowResult<Option<Receipt>> {
    
    let client = Client::from_env(risc0_zkvm::VERSION)?;

    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let execute_only = false;
    
    

    //TODO: store in DB
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    println!("sessionId: {:?}", session.uuid);

    update_session_id_superproof(get_pool().await,  &session.uuid, superproof_id);

    let receipt = check_session_status(session, client).await?;    
    Ok(receipt)
}

async fn check_session_status(session: SessionId, client: Client) -> AnyhowResult<Option<Receipt>> {
    let mut receipt: Option<Receipt> = None;
    loop {
        let res = session.status(&client).await?;
        // TODO: store Risc0 status in DB
        if res.status == "RUNNING" {
            println!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // TODO: store Risc0 status in DB
            println!("proof reduction completed");
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = client.download(&receipt_url).await?;
            receipt = Some(bincode::deserialize(&receipt_buf)?);
            // receipt
            //     .verify(METHOD_ID)
            //     .expect("Receipt verification failed");

            
        } else {
            println!("inside else");
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
        break;
    }
    Ok(receipt)
}
