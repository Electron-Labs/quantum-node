use std::time::Duration;

use bonsai_sdk::{non_blocking::{Client, SessionId}, responses::SnarkReceipt};
use quantum_db::repository::{bonsai_image::get_bonsai_image_by_image_id, proof_repository::update_session_id_in_proof, superproof_repository::{update_session_id_superproof, update_snark_session_id_superproof}};
use quantum_utils::error_line;
use risc0_zkvm::Receipt;
use tracing::{info, error};

use crate::{connection::get_pool, worker::increment_cycle};

use anyhow::{anyhow, Result as AnyhowResult};
pub async fn execute_proof_reduction(input_data: Vec<u8>, image_id: &str, proof_id: u64) -> AnyhowResult<(Option<Receipt>, String)> {
    
    let client = Client::from_env(risc0_zkvm::VERSION)?;

    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let bonsai_image = get_bonsai_image_by_image_id(get_pool().await, image_id).await?;

    let assumptions: Vec<String> = vec![];

    // Wether to run in execute only mode
    let execute_only = false;

    //TODO: store in DB
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    let session_uuid_id = session.uuid.clone();
    println!("sessionId: {:?}", session.uuid);

    update_session_id_in_proof(get_pool().await, proof_id, &session.uuid).await?;

    let receipt = check_session_status(session, client, &bonsai_image.circuit_verifying_id).await?;
    Ok((receipt, session_uuid_id))
}

pub async fn execute_aggregation(input_data: Vec<u8>, image_id: &str, assumptions: Vec<String>, superproof_id: u64, ) -> AnyhowResult<(Option<Receipt>, String)> {
    
    let client = Client::from_env(risc0_zkvm::VERSION)?;
    let bonsai_image = get_bonsai_image_by_image_id(get_pool().await, image_id).await?;
    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let execute_only = false;
    
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    let session_uuid_id = session.uuid.clone();
    println!("sessionId: {:?}", session.uuid);
    update_session_id_superproof(get_pool().await,  &session.uuid, superproof_id).await?;

    let receipt = check_session_status(session, client, &bonsai_image.circuit_verifying_id).await?;    
    Ok((receipt, session_uuid_id))
}

async fn check_session_status(session: SessionId, client: Client, circuit_verifying_id: &[u32;8] ) -> AnyhowResult<Option<Receipt>> {
    let mut receipt: Option<Receipt> = None;
    loop {
        let res = session.status(&client).await?;
        // TODO: store Risc0 status in DB
        if res.status == "RUNNING" {
            info!(
                "Current status for session_id {} : {} - state: {} - continue polling...",
                session.uuid,
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // TODO: store Risc0 status in DB
            info!("proof reduction completed for session_id: {:?}", session.uuid);
            
            //TODO: remove the unwrap
            let cycle_used = res.stats.unwrap().cycles;
            increment_cycle(cycle_used as i64).await;
            
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = client.download(&receipt_url).await?;
            receipt = Some(bincode::deserialize(&receipt_buf)?);
            // let METHOD_ID = get_verifier_id(0);
            receipt.clone().unwrap().verify(circuit_verifying_id.clone())
                .expect("Receipt verification failed");

            
        } else {
            info!("sesion status: {:?}", res.status);
            error!("error occured in bonsai session: {:?} with status {:?}, and error messgae: {:?}", &session.uuid, res.status, res.error_msg);
            return Err(anyhow!(error_line!("bonsai_session_failed")));
            // panic!(
            //     "Workflow exited: {} - | err: {}",
            //     res.status,
            //     res.error_msg.unwrap_or_default()
            // );
            // error!("session status: {:?}, wit")
        }
        break;
    }
    Ok(receipt)
}


pub async fn run_stark2snark(agg_session_id: String, superproof_id: u64) -> AnyhowResult<Option<SnarkReceipt>> {
    let client = Client::from_env(risc0_zkvm::VERSION)?;
    let mut receipt: Option<SnarkReceipt> = None;
    let snark_session = client.create_snark(agg_session_id).await?;
    println!("Created snark session: {}", snark_session.uuid);
    update_snark_session_id_superproof(get_pool().await, &snark_session.uuid, superproof_id).await?;
    loop {
        let res = snark_session.status(&client).await?;
        match res.status.as_str() {
            "RUNNING" => {
                println!("Current status: {} - continue polling...", res.status,);
                std::thread::sleep(Duration::from_secs(15));
                continue;
            }
            "SUCCEEDED" => {
                let snark_receipt = res.output;
                println!("Snark proof!: {snark_receipt:?}");
                // let file = File::create("snark_receipt.json").unwrap();
                // let mut writer = BufWriter::new(file);
                // serde_json::to_writer(&mut writer, &snark_receipt.unwrap()).unwrap();
                receipt = snark_receipt;
                break;
            }
            _ => {
                // panic!(
                //     "Workflow exited: {} err: {}",
                //     res.status,
                //     res.error_msg.unwrap_or_default()
                // );
                error!("error occured in bonsai session: {:?} with status {:?}, and error messgae: {:?}", &snark_session.uuid, res.status, res.error_msg);
                return Err(anyhow!(error_line!("bonsai_session_failed")));
            }
        }
    }
    Ok(receipt)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv::dotenv;
//     use quantum_db::repository::proof_repository::get_proofs_in_superproof_id;

//     #[tokio::test]
//     #[ignore]
//     pub async fn test_start_to_snark() {
//         // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
//         // dotenv::from_filename("../.env.test").ok();
//         dotenv().ok();
//         let session_id = "090c5ffa-3ed1-4bc5-a430-d6fb5d32d969";
//         run_stark2snark(session_id.to_string()).await;
//     }
// }