pub mod connection;
// use std::{thread, time::{Duration, Instant}};

use chrono::{DateTime, Utc};
use connection::get_pool;
use dotenv::dotenv;
use quantum_db::repository::superproof_repository::{get_first_non_submitted_superproof, get_last_verified_superproof, update_superproof_onchain_submission_time};
use quantum_utils::logger::initialize_logger;

use anyhow::{anyhow, Ok, Result as AnyhowResult};
use sqlx::types::chrono::NaiveDateTime;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

const SUPERPROOF_SUBMISSION_DURATION: u64 = 60*60;
const SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED: u64 = 5*60;

async fn initialize_superproof_submission_loop(superproof_submission_duration: Duration) -> AnyhowResult<()> {
    loop {
        info!("----checking for new superproof to submit----");
        let last_verified_superproof = get_last_verified_superproof(get_pool().await).await?;
        let last_verified_superproof = match last_verified_superproof {
            Some(superproof) => superproof,
            None => {
                info!("no verified superproof found.");
                info!("sleep for {:?}", SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED);
                sleep(Duration::from_secs(SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED)).await;
                continue;
            }
        };

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

        let new_superproof_id = match first_superproof_not_verfied.id {
            Some(id) => Ok(id),
            None => Err(anyhow!("id of the new superproof not present")),
        }?;

    
        let current_time = get_current_time();
        update_superproof_onchain_submission_time(get_pool().await, current_time, new_superproof_id).await?;
        
        // let gas_cost = get_gas_cost().await.unwrap();
        // let eth_price = get_eth_price().await.unwrap();
        //make smart contract call here

        // update the transaction hash and gas cost


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
    dotenv().ok();
    println!(" --- Starting worker --- ");
    let _guard = initialize_logger("qunatum_contract.log");
    let superproof_submission_duration = Duration::from_secs(SUPERPROOF_SUBMISSION_DURATION);
    let error = initialize_superproof_submission_loop(superproof_submission_duration).await;
    let err = error.expect_err("No error should not return");
    error!("Some error occured in quantum contract: {:?}", err);
}
