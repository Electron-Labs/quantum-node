pub mod connection;
pub mod contract;
pub mod contract_utils;
pub mod quantum_contract;

use chrono::{DateTime, Utc};
use connection::get_pool;
use contract::gen_quantum_structs;
use dotenv::dotenv;
use ethers::utils::hex::ToHexExt;
use quantum_contract::{Batch, Protocol};
// use ethers::utils::hex::traits::ToHex;
use keccak_hash::keccak;
use quantum_db::repository::{
    proof_repository::{get_proofs_in_superproof_id, update_proof_status},
    reduction_circuit_repository::get_reduction_circuit_for_user_circuit,
    superproof_repository::{
        get_first_non_submitted_superproof, get_last_verified_superproof,
        update_superproof_fields_after_onchain_submission,
        update_superproof_onchain_submission_time,
    },
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::enums::proving_schemes::ProvingSchemes;
use quantum_types::traits::pis;
use quantum_types::{
    enums::{proof_status::ProofStatus, superproof_status::SuperproofStatus},
    traits::{pis::Pis, proof::Proof},
    types::{
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof},
        snarkjs_groth16::SnarkJSGroth16Pis,
    },
};
use quantum_utils::logger::initialize_logger;

use anyhow::{anyhow, Result as AnyhowResult};
use sqlx::types::chrono::NaiveDateTime;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use crate::{
    contract::{get_quantum_contract, update_quantum_contract_state},
    contract_utils::{get_eth_price, get_gas_cost},
};

const SUPERPROOF_SUBMISSION_DURATION: u64 = 15 * 60;
const SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED: u64 = 30;

async fn initialize_superproof_submission_loop(
    superproof_submission_duration: Duration,
) -> AnyhowResult<()> {
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
            let remaining_time = SUPERPROOF_SUBMISSION_DURATION as i64 - time_elapsed.num_seconds();
            if remaining_time > 0 {
                info!(
                    "The remaining for new superproof submission : {:?}",
                    remaining_time
                );
                info!("Sleeping for {:?}", remaining_time);
                sleep(Duration::from_secs(remaining_time as u64)).await;
                continue;
            }
        }

        let first_superproof_not_verfied =
            get_first_non_submitted_superproof(get_pool().await).await?;
        let first_superproof_not_verfied = match first_superproof_not_verfied {
            Some(superproof) => superproof,
            None => {
                info!("No new provingDone superproof find");
                info!(
                    "Sleeping for {:?}",
                    SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED
                );
                sleep(Duration::from_secs(
                    SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED,
                ))
                .await;
                continue;
            }
        };

        let superproof_proof_path = first_superproof_not_verfied.superproof_proof_path.unwrap();
        let gnark_proof = GnarkGroth16Proof::read_proof(&superproof_proof_path)?;

        let new_superproof_id = match first_superproof_not_verfied.id {
            Some(id) => Ok(id),
            None => Err(anyhow!("id of the new superproof not present")),
        }?;

        let proofs = get_proofs_in_superproof_id(get_pool().await, new_superproof_id).await?;

        let mut protocols = [Protocol::default(); 10];
        for (i, proof) in proofs.clone().iter().enumerate() {
            let user_circuit =
                get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash)
                    .await?;

            // compute pis_hash
            let pis_hash: [u8; 32];
            match user_circuit.proving_scheme {
                ProvingSchemes::GnarkGroth16 => {
                    pis_hash = GnarkGroth16Pis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                ProvingSchemes::Groth16 => {
                    pis_hash = SnarkJSGroth16Pis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                _ => todo!(),
            }

            // compute vk_hash
            let protocol_vk_hash = hex::decode(&proof.user_circuit_hash[2..])?;
            let reduction_vk_hash = hex::decode(&user_circuit.reduction_circuit_id.unwrap()[2..])?;
            let concat = [protocol_vk_hash, reduction_vk_hash].concat();
            let vk_hash = keccak(concat).0;

            protocols[i] = Protocol {
                vk_hash,
                pub_inputs_hash: pis_hash,
            };
        }

        let quantum_contract = get_quantum_contract()?;

        let current_time = get_current_time();
        update_superproof_onchain_submission_time(
            get_pool().await,
            current_time,
            new_superproof_id,
        )
        .await?;

        let gas_cost = get_gas_cost().await.unwrap();
        let eth_price = get_eth_price().await.unwrap();

        //make smart contract call here
        let receipt =
            update_quantum_contract_state(&quantum_contract, Batch { protocols }, &gnark_proof)
                .await?;
        let receipt = match receipt {
            Some(r) => {
                println!("succefully updated quantum contract!");
                Ok(r)
            },
            None => Err(anyhow!("error in updating the quantum contract")),
        }?;

        // update the transaction_hash, gas_cost, eth_price and gas cost
        let transaction_hash = receipt.transaction_hash.encode_hex();
        let transaction_hash = String::from("0x") + &transaction_hash;

        for proof in proofs {
            update_proof_status(get_pool().await, &proof.proof_hash, ProofStatus::Verified).await?;
        }

        update_superproof_fields_after_onchain_submission(
            get_pool().await,
            &transaction_hash,
            gas_cost,
            eth_price,
            SuperproofStatus::SubmittedOnchain,
            new_superproof_id,
        )
        .await?;

        info!("Sleeping for {:?}", superproof_submission_duration);
        sleep(superproof_submission_duration).await;
    }
}

fn get_current_time() -> NaiveDateTime {
    let now_utc: DateTime<Utc> = Utc::now();
    now_utc.naive_utc()
}

#[tokio::main]
async fn main() {
    // gen_quantum_structs().unwrap();
    dotenv().ok();
    info!(" --- Starting quantum contract --- ");
    let _guard = initialize_logger("quantum_contract.log");
    let _db_pool = get_pool().await;
    let superproof_submission_duration = Duration::from_secs(SUPERPROOF_SUBMISSION_DURATION);
    let error = initialize_superproof_submission_loop(superproof_submission_duration).await;
    let err = error.expect_err("No error should not return");
    error!("Some error occured in quantum contract: {:?}", err);
}
