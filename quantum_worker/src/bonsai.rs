use std::{fs, time::Duration};

use bonsai_sdk::non_blocking::{Client, SessionId};
use quantum_db::repository::{bonsai_image::get_bonsai_image_by_image_id, proof_repository::{update_cycle_used_in_proof, update_session_id_in_proof}, superproof_repository::{update_session_id_superproof, update_snark_session_id_superproof}};
use quantum_utils::error_line;
use risc0_zkvm::{default_prover, ExecutorEnv, Receipt};
use tracing::{info, error};

use crate::{connection::get_pool, worker::increment_cycle};

use anyhow::{anyhow, Result as AnyhowResult};

pub fn get_bonsai_client() -> AnyhowResult<Client> {
    let client = Client::from_env(risc0_zkvm::VERSION)?;
    Ok(client)
}

pub async fn execute_proof_reduction(input_data: Vec<u8>, image_id: &str, proof_id: u64, assumptions: Vec<String>) -> AnyhowResult<(Option<Receipt>, String)> {
    
    let client = get_bonsai_client()?;

    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let bonsai_image = get_bonsai_image_by_image_id(get_pool().await, image_id).await?;

    // Wether to run in execute only mode
    let execute_only = false;

    //TODO: store in DB
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    let session_uuid_id = session.uuid.clone();
    println!("sessionId: {:?}", session.uuid);

    update_session_id_in_proof(get_pool().await, proof_id, &session.uuid).await?;

    let (receipt, cycle_used) = check_session_status(session, client, &bonsai_image.circuit_verifying_id).await?;

    update_cycle_used_in_proof(get_pool().await, proof_id, cycle_used).await?;

    Ok((receipt, session_uuid_id))
}

pub async fn execute_aggregation(input_data: Vec<u8>, image_id: &str, assumptions: Vec<String>, superproof_id: u64, ) -> AnyhowResult<(Option<Receipt>, String, u64)> {
    
    let client = get_bonsai_client()?;
    let bonsai_image = get_bonsai_image_by_image_id(get_pool().await, image_id).await?;
    // TODO: store it in DB
    let input_id = client.upload_input(input_data).await?;
    println!("input_id: {:?}", input_id);

    let execute_only = false;
    
    let session = client.create_session(image_id.to_string(), input_id, assumptions, execute_only).await?;
    let session_uuid_id = session.uuid.clone();
    println!("sessionId: {:?}", session.uuid);
    update_session_id_superproof(get_pool().await,  &session.uuid, superproof_id).await?;

    let (receipt, cycle_used )= check_session_status(session, client, &bonsai_image.circuit_verifying_id).await?;    
    Ok((receipt, session_uuid_id, cycle_used))
}

async fn check_session_status(session: SessionId, client: Client, circuit_verifying_id: &[u32;8] ) -> AnyhowResult<(Option<Receipt>, u64)> {
    let mut receipt: Option<Receipt> = None;
    loop {
        let res = session.status(&client).await?;
        println!("Current status: {}", res.status);
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
            info!("cycle used in session {:?}: {:?}", &session.uuid, cycle_used);
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
            return Ok((receipt, cycle_used));
            
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
        // break;
    }
    // Ok(receipt)
}


pub async fn run_stark2snark(agg_session_id: String, superproof_id: u64) -> AnyhowResult<Option<Receipt>> {
    let client = get_bonsai_client()?;
    let mut receipt: Option<Receipt> = None;
    let snark_session = client.create_snark(agg_session_id).await?;
    println!("Created snark session: {}", snark_session.uuid);
    update_snark_session_id_superproof(get_pool().await, &snark_session.uuid, superproof_id).await?;
    loop {
        let res = snark_session.status(&client).await.map_err(|err| anyhow!(error_line!(err)))?;
        println!("Current status: {}", res.status,);
        match res.status.as_str() {
            "RUNNING" => {
                println!("continue polling...");
                std::thread::sleep(Duration::from_secs(15));
                continue;
            }
            "SUCCEEDED" => {
                let snark_receipt_url = res.output.unwrap();

                let receipt_buf = client.download(&snark_receipt_url).await?;
                let snark_receipt: Receipt  = bincode::deserialize(&receipt_buf)?;
                println!("Snark proof!: {snark_receipt:?}");
                // let file = File::create("snark_receipt.json").unwrap();
                // let mut writer = BufWriter::new(file);
                // serde_json::to_writer(&mut writer, &snark_receipt.unwrap()).unwrap();
                receipt = Some(snark_receipt);
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


pub async fn upload_receipt(receipt: Receipt) -> AnyhowResult<String> {
    let client = get_bonsai_client()?;
    let serialized_receipt = bincode::serialize(&receipt)?;
    let receipt_id = client.upload_receipt(serialized_receipt).await?;
    Ok(receipt_id)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use dotenv::dotenv;
//     use quantum_db::repository::proof_repository::get_proofs_in_superproof_id;

//     // #[tokio::test]
//     // #[ignore]
//     // #[test]
//     // pub fn test_start_to_snark() {
//     //     let bytes = fs::read("./storage/0xceeb414032c1ce1d0d9e8627bf132e49c6e528f2c60c8c8d12890d99ffdaecc3/receipt/reduced_proof_receipt_0xb50ea463d922cc7aad3d1e420782f412a44d80039cb915e89d45189469e1be6b.bin").unwrap();
//     //     let receipt: Receipt = serde_json::from_slice(&bytes).unwrap();

//     //     let image_id = [4041203892,1423599498,480885055,2897245618,1324039803,3635355280,3442530142,2448524712];

//     //     risc0_execute(receipt, image_id).unwrap();

//     // }
// }