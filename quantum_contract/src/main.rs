pub mod connection;
pub mod contract;
pub mod contract_utils;
pub mod quantum_contract;

use chrono::{DateTime, Utc};
use connection::get_pool;
use dotenv::dotenv;
use ethers::utils::hex::ToHexExt;
// use ethers::utils::hex::traits::ToHex;
use quantum_db::repository::superproof_repository::{get_first_non_submitted_superproof, get_last_verified_superproof, update_superproof_fields_after_onchain_submission, update_superproof_onchain_submission_time};
use quantum_types::{enums::superproof_status::SuperproofStatus, traits::proof::Proof, types::gnark_groth16::GnarkGroth16Proof};
use quantum_utils::logger::initialize_logger;

use anyhow::{anyhow, Result as AnyhowResult};
use sqlx::types::chrono::NaiveDateTime;
use tokio::time::{sleep, Duration};
use tracing::{error, info};


use crate::{contract::{get_quantum_contract, update_quantum_contract_state}, contract_utils::{get_eth_price, get_gas_cost}};

const SUPERPROOF_SUBMISSION_DURATION: u64 = 60*60;
const SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED: u64 = 30;

async fn initialize_superproof_submission_loop(superproof_submission_duration: Duration) -> AnyhowResult<()> {
    loop {
        info!("----checking for new superproof to submit----");
        let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
        // let last_verified_superproof = match last_verified_superproof {
        //     Some(superproof) => superproof,
        //     None => {
        //         info!("no verified superproof found.");
        //         // info!("sleep for {:?}", SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED);
        //         // sleep(Duration::from_secs(SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED)).await;
        //         // continue;
        //         handle_proof_submission()
        //     }
        // };
        if last_verified_superproof.is_some() {
            let last_verified_superproof = match last_verified_superproof {
                Some(superproof) => Ok(superproof),
                None => Err(anyhow!("error in getting last superproof verified")),
            }?;
            let on_chain_submission_time = match last_verified_superproof.onchain_submission_time {
                Some(t) => Ok(t),
                None => Err(anyhow!("submitted superproof dont have timestamp")),
            }?;

            let current_time = get_current_time();
            let time_elapsed = current_time - on_chain_submission_time;
            let remaining_time = 60*60 - time_elapsed.num_seconds();
            if remaining_time > 0 {
                info!("The remaining for new superproof submission : {:?}", remaining_time);
                info!("Sleeping for {:?}", remaining_time);
                sleep(Duration::from_secs(remaining_time as u64)).await;
                continue;
            }
        }

        let first_superproof_not_verfied = get_first_non_submitted_superproof(get_pool().await).await?;
        let first_superproof_not_verfied = match first_superproof_not_verfied {
            Some(superproof) => superproof,
            None => {
                info!("No new provingDone superproof find");
                info!("Sleeping for {:?}", SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED);
                sleep(Duration::from_secs(SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED)).await;
                continue;
            }
        };
        
        let new_root = first_superproof_not_verfied.superproof_root.unwrap();
        
        let superproof_proof_path = first_superproof_not_verfied.superproof_proof_path.unwrap();
        let gnark_proof = GnarkGroth16Proof::read_proof(&superproof_proof_path)?;


        let new_superproof_id = match first_superproof_not_verfied.id {
            Some(id) => Ok(id),
            None => Err(anyhow!("id of the new superproof not present")),
        }?;

    
        let quantum_contract = get_quantum_contract()?;

        let current_time = get_current_time();
        update_superproof_onchain_submission_time(get_pool().await, current_time, new_superproof_id).await?;
        
        let gas_cost = get_gas_cost().await.unwrap();
        let eth_price = get_eth_price().await.unwrap();
        
        //make smart contract call here
        let receipt = update_quantum_contract_state(&quantum_contract, &new_root, &gnark_proof).await?;
        let receipt = match receipt {
            Some(r) => Ok(r),
            None => Err(anyhow!("error in updating the quantum contract")),
        }?;

        // update the transaction_hash, gas_cost, eth_price and gas cost
        let transaction_hash = receipt.transaction_hash.encode_hex();
        let transaction_hash = String::from("0x") + &transaction_hash;

        update_superproof_fields_after_onchain_submission(get_pool().await, &transaction_hash, gas_cost, eth_price, SuperproofStatus::SubmittedOnchain, new_superproof_id).await?;

        info!("Sleeping for {:?}", superproof_submission_duration);
        sleep(superproof_submission_duration).await;
    }
}

fn get_current_time() -> NaiveDateTime{
    let now_utc: DateTime<Utc> = Utc::now();
    now_utc.naive_utc()
}


#[tokio::main]
async fn main() {
    // gen_quantum_structs().unwrap();
    dotenv().ok();
    println!(" --- Starting quantum contract --- ");
    let _guard = initialize_logger("quantum_contract.log");
    let _db_pool = get_pool().await;
    let superproof_submission_duration = Duration::from_secs(SUPERPROOF_SUBMISSION_DURATION);
    let error = initialize_superproof_submission_loop(superproof_submission_duration).await;
    let err = error.expect_err("No error should not return");
    error!("Some error occured in quantum contract: {:?}", err);
}
