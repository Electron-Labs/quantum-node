pub mod connection;
pub mod contract;
pub mod contract_utils;
pub mod quantum_contract;

use std::time::Instant;

use chrono::{DateTime, Utc};
use connection::get_pool;
use contract::{gen_quantum_structs, register_cricuit_in_contract};
use dotenv::dotenv;
use ethers::{etherscan::gas, types::TransactionReceipt, utils::hex::ToHexExt};
use quantum_contract::{Batch, Protocol};
// use ethers::utils::hex::traits::ToHex;
use keccak_hash::keccak;
use quantum_db::repository::{
    cost_saved_repository::insert_cost_saved_data, 
    proof_repository::{get_proofs_in_superproof_id, update_proof_status}, 
    reduction_circuit_repository::get_reduction_circuit_for_user_circuit, 
    superproof_repository::{
        get_first_non_submitted_superproof, get_last_verified_superproof,
        update_superproof_fields_after_onchain_submission,
        update_superproof_onchain_submission_time,
    }, 
    user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, get_user_circuits_by_circuit_status, update_user_circuit_data_reduction_status},
};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes}, types::halo2_plonk::Halo2PlonkPis};
use quantum_types::traits::pis;
use quantum_types::{
    enums::{proof_status::ProofStatus, superproof_status::SuperproofStatus},
    traits::{pis::Pis, proof::Proof},
    types::{
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof},
        snarkjs_groth16::SnarkJSGroth16Pis,
    },
};
use quantum_utils::{error_line, logger::initialize_logger};

use anyhow::{anyhow, Error, Result as AnyhowResult};
use sqlx::types::chrono::NaiveDateTime;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use crate::{
    contract::{get_quantum_contract, update_quantum_contract_state},
    contract_utils::{get_eth_price, get_gas_cost},
};

const SUPERPROOF_SUBMISSION_DURATION: u64 = 20 * 60;
const SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED: u64 = 30;
const REGISTER_CIRCUIT_LOOP_DURATION: u64 = 1*60;
const RETRY_COUNT: u64 = 3;
const TOTAL_GAS_USED_WITHOUT_QUANTUM: u64 = 350_000 * 10;

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

        let mut protocols = [Protocol::default(); 20];
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
                ProvingSchemes::Halo2Plonk => {
                    pis_hash = Halo2PlonkPis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                _ => {
                    error!("{:?}",error_line!("unsupoorted proving scheme"));
                    panic!("due to unsupported proving scheme");
                },
            }

            // compute vk_hash
            let vk_hash = get_vk_hash_for_smart_contract(user_circuit.circuit_hash, user_circuit.reduction_circuit_id.unwrap())?;

            protocols[i] = Protocol {
                vk_hash,
                pub_inputs_hash: pis_hash,
            };
        }



        let current_time = get_current_time();
        update_superproof_onchain_submission_time(
            get_pool().await,
            current_time,
            new_superproof_id,
        )
        .await?;

        let (transaction_hash, gas_used) = make_smart_contract_call_with_retry(protocols, &gnark_proof).await?;

        let gas_cost = get_gas_cost().await?;
        let eth_price = get_eth_price().await?;

        let total_cost_usd = calc_total_cost_usd(gas_used, gas_cost, eth_price);

        for proof in proofs {
            update_proof_status(get_pool().await, &proof.proof_hash, ProofStatus::Verified).await?;
        }

        update_superproof_fields_after_onchain_submission(
            get_pool().await,
            &transaction_hash,
            gas_cost,
            eth_price,
            SuperproofStatus::SubmittedOnchain,
            total_cost_usd,
            gas_used,
            new_superproof_id,
        )
        .await?;

        let total_gas_saved = TOTAL_GAS_USED_WITHOUT_QUANTUM - gas_used;
        let total_usd_saved = calc_total_cost_usd(total_gas_saved, gas_cost, eth_price);

        insert_cost_saved_data(get_pool().await, total_gas_saved, total_usd_saved).await?;

        info!("Sleeping for {:?}", superproof_submission_duration);
        sleep(superproof_submission_duration).await;
    }
}

async fn make_smart_contract_call_with_retry(protocols: [Protocol; 10], gnark_proof: &GnarkGroth16Proof) -> AnyhowResult<(String, u64)> {
    let mut retry_count = 0;
    let transaction_hash;
    let quantum_contract = get_quantum_contract()?;
    let gas_used;
    let mut error = Err(anyhow!(error_line!("Error initialized")));
    while retry_count <= RETRY_COUNT {
        match update_quantum_contract_state(&quantum_contract, Batch { protocols }, &gnark_proof).await {
            Ok(receipt) =>{
                let transaction_hash_string = receipt.transaction_hash.encode_hex();
                let transaction_hash_string = String::from("0x") + &transaction_hash_string;
                transaction_hash = transaction_hash_string;
                gas_used = receipt.gas_used.ok_or_else(|| anyhow::anyhow!("Gas used is not found"))?.as_u64();
                return Ok((transaction_hash, gas_used));
            }
            Err(e) => {
                retry_count = retry_count+1;
                error!("error occured in smart contract call, retrying count {:?}, error: {:?}",retry_count, error_line!(e));
                error =  Err(anyhow!(error_line!(e)));
            },
        }
    }
    error
}

fn calc_total_cost_usd(gas_used: u64, gas_cost: f64, eth_price: f64) -> f64 {
    (gas_used as f64 * gas_cost * eth_price)/ 1e9
}

fn get_current_time() -> NaiveDateTime {
    let now_utc: DateTime<Utc> = Utc::now();
    now_utc.naive_utc()
}

fn get_vk_hash_for_smart_contract(user_circuit_hash: String, reduction_circuit_hash: String) -> AnyhowResult<[u8;32]> {
    let protocol_vk_hash = hex::decode(user_circuit_hash[2..].to_string()).map_err(|err| anyhow!(error_line!(err)))?;
    let reduction_vk_hash = hex::decode(reduction_circuit_hash[2..].to_string())?;
    let concat = [protocol_vk_hash, reduction_vk_hash].concat();
    let vk_hash = keccak(concat).0;
    Ok(vk_hash)
}

async fn initialize_circuit_registration_loop() -> AnyhowResult<()> {
    info!("----starting cirucit registration loop----");
    loop {
        let user_circuit_not_registered = get_user_circuits_by_circuit_status(get_pool().await, CircuitReductionStatus::SmartContractRgistrationPending).await?;
        let quantum_contract = get_quantum_contract()?;
        for user_circuit in user_circuit_not_registered {
            info!("calculating vk_hash for circuit hash: {:?}", user_circuit.circuit_hash);
            let vk_hash = get_vk_hash_for_smart_contract(user_circuit.circuit_hash.clone(), user_circuit.reduction_circuit_id.unwrap())?;
            register_cricuit_in_contract(vk_hash, &quantum_contract).await?;
            update_user_circuit_data_reduction_status(get_pool().await, &user_circuit.circuit_hash, CircuitReductionStatus::Completed).await?;
        }
        sleep(Duration::from_secs(REGISTER_CIRCUIT_LOOP_DURATION)).await;
    }
}
 
#[tokio::main]
async fn main() {
    // gen_quantum_structs().unwrap();
    dotenv().ok();
    info!(" --- Starting quantum contract --- ");
    let _guard = initialize_logger("quantum_contract.log");
    let _db_pool = get_pool().await;
    let superproof_submission_duration = Duration::from_secs(SUPERPROOF_SUBMISSION_DURATION);

    let task2 = tokio::spawn(async move {
        match initialize_superproof_submission_loop(superproof_submission_duration).await {
            Ok(_) => info!("contract poller exit without any error"),
            Err(e) => error!("contract poller exit with error: {:?}", e.root_cause().to_string()),
        };
    });

    let task1 = tokio::spawn(async move {
        match initialize_circuit_registration_loop().await {
            Ok(_) => info!("register circuit loop exit without any error"),
            Err(e) => error!("register circuit loop exit with error: {:?}", e.root_cause().to_string()),
        };
    });

    tokio::join!(task1, task2);
    
}
