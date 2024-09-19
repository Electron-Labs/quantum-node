pub mod connection;
pub mod contract;
pub mod contract_utils;
pub mod quantum_contract;

use std::time::Instant;

use chrono::{DateTime, Utc};
use connection::get_pool;
use contract::{gen_quantum_structs, register_cricuit_in_contract};
use contract_utils::get_bytes_from_hex_string;
use dotenv::dotenv;
use ethers::{etherscan::gas, types::TransactionReceipt, utils::hex::ToHexExt};
use quantum_contract::{Protocol, TreeUpdate};
// use ethers::utils::hex::traits::ToHex;
use keccak_hash::keccak;
use quantum_db::repository::{
    cost_saved_repository::udpate_cost_saved_data,
    proof_repository::{get_proofs_in_superproof_id, update_proof_status},
    // reduction_circuit_repository::get_reduction_circuit_for_user_circuit,
    superproof_repository::{
        get_first_non_submitted_superproof, get_last_verified_superproof, update_superproof_fields_after_onchain_submission, update_superproof_gas_data, update_superproof_onchain_submission_time
    },
    user_circuit_data_repository::{get_user_circuit_data_by_circuit_hash, get_user_circuits_by_circuit_status, update_user_circuit_data_reduction_status},
};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proving_schemes::ProvingSchemes}, types::{gnark_plonk::GnarkPlonkPis, halo2_plonk::Halo2PlonkPis}};
use quantum_types::traits::pis;
use quantum_types::{
    enums::{proof_status::ProofStatus, superproof_status::SuperproofStatus},
    traits::{pis::Pis, proof::Proof},
    types::{
        gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof},
        snarkjs_groth16::SnarkJSGroth16Pis,
    },
};
use quantum_utils::{error_line, keccak::decode_keccak_hex, logger::initialize_logger};

use anyhow::{anyhow, Error, Result as AnyhowResult};
use sqlx::types::chrono::NaiveDateTime;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

use crate::{
    contract::{get_quantum_contract, update_quantum_contract_state},
    contract_utils::{get_eth_price, get_gas_cost},
};

const SUPERPROOF_SUBMISSION_RETRY: u64 = 5 * 60;
const SUPERPROOF_SUBMISSION_DURATION: u64 = 25 * 60;
const SLEEP_DURATION_WHEN_NEW_SUPERPROOF_IS_NOT_VERIFIED: u64 = 30;
const REGISTER_CIRCUIT_LOOP_DURATION: u64 = 1*60;
const RETRY_COUNT: u64 = 3;
const DIRECT_PROOF_VERIFICATION_GAS_COST: u64 = 350_000;


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

        let mut protocols = vec![];
        for (i, proof) in proofs.clone().iter().enumerate() {
            let user_circuit =
                get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash)
                    .await?;

            // compute pis_hash
            let pis_hash: [u8; 32];
            match user_circuit.proving_scheme {
                ProvingSchemes::GnarkGroth16 => {
                    pis_hash = GnarkGroth16Pis::read_pis(&proof.pis_path)?.keccak_hash()?;
                }
                ProvingSchemes::Groth16 => {
                    pis_hash = SnarkJSGroth16Pis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                ProvingSchemes::Halo2Plonk => {
                    pis_hash = Halo2PlonkPis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                ProvingSchemes::GnarkPlonk => {
                    pis_hash = GnarkPlonkPis::read_pis(&proof.pis_path)?.keccak_hash()?
                }
                _ => {
                    error!("{:?}",error_line!("unsupoorted proving scheme"));
                    panic!("due to unsupported proving scheme");
                },
            }

            protocols.push(Protocol {
                combined_vkey_hash: decode_keccak_hex(&user_circuit.circuit_hash)?,
                pis_hash,
            });
        }

        let new_root =
            get_bytes_from_hex_string(&first_superproof_not_verfied.superproof_root.ok_or(
                anyhow!(error_line!(
                    "missing first_superproof_not_verfied.superproof_root"
                )),
            )?)?;

        let current_time = get_current_time();
        update_superproof_onchain_submission_time(
            get_pool().await,
            current_time,
            new_superproof_id,
        )
        .await?;

        let (transaction_hash, gas_used) = make_smart_contract_call_with_retry(protocols.clone(), new_root, &gnark_proof).await?;

        // update tx data in DB for superproof
        update_superproof_fields_after_onchain_submission(
            get_pool().await,
            &transaction_hash,
            SuperproofStatus::SubmittedOnchain,
            gas_used,
            new_superproof_id,
        )
        .await?;

        // TODO: remove unwrap
        for proof in proofs {
            update_proof_status(get_pool().await, proof.id.unwrap(), ProofStatus::Verified).await?;
        }

        // update gas data in DB for superproof
        let gas_cost = get_gas_cost().await?;
        let eth_price = get_eth_price().await?;
        let total_cost_usd = calc_total_cost_usd(gas_used, gas_cost, eth_price);
        update_superproof_gas_data(
            get_pool().await,
            gas_cost,
            eth_price,
            total_cost_usd,
            new_superproof_id,
        )
        .await?;

        let total_gas_saved_batch = (DIRECT_PROOF_VERIFICATION_GAS_COST * 20) - gas_used;
        let total_usd_saved_batch = calc_total_cost_usd(total_gas_saved_batch, gas_cost, eth_price);
        udpate_cost_saved_data(get_pool().await, total_gas_saved_batch, total_usd_saved_batch).await?;

        info!("Sleeping for {:?}", superproof_submission_duration);
        sleep(superproof_submission_duration).await;
    }
}

async fn make_smart_contract_call_with_retry(protocols: Vec<Protocol>, new_root: [u8; 32], gnark_proof: &GnarkGroth16Proof) -> AnyhowResult<(String, u64)> {
    let mut retry_count = 0;
    let transaction_hash;
    let quantum_contract = get_quantum_contract()?;
    let gas_used;
    let mut error = Err(anyhow!(error_line!("Error initialized")));
    while retry_count <= RETRY_COUNT {
        println!("gnark_proof {:?}", gnark_proof);
        println!("new_root {:?}", new_root);
        println!("tree_root {:?}", quantum_contract.tree_root().await?);
        match update_quantum_contract_state(&quantum_contract, protocols.clone(), TreeUpdate { new_root }, &gnark_proof).await {
            Ok(receipt) =>{
                let transaction_hash_string = receipt.transaction_hash.encode_hex();
                let transaction_hash_string = String::from("0x") + &transaction_hash_string;
                transaction_hash = transaction_hash_string;
                gas_used = receipt.gas_used.ok_or_else(|| anyhow::anyhow!("Gas used is not found"))?.as_u64();
                return Ok((transaction_hash, gas_used));
            }
            Err(e) => {
                retry_count = retry_count+1;
                info!("error occured in smart contract call, retrying count {:?}, error: {:?}",retry_count, error_line!(e));
                info!("Trying again in 10 seconds");
                sleep(Duration::from_secs(10)).await;
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

async fn initialize_circuit_registration_loop() -> AnyhowResult<()> {
    info!("----starting cirucit registration loop----");
    loop {
        let user_circuit_not_registered = get_user_circuits_by_circuit_status(get_pool().await, CircuitReductionStatus::SmartContractRgistrationPending).await?;
        let quantum_contract = get_quantum_contract()?;
        for user_circuit in user_circuit_not_registered {
            info!("calculating vk_hash for circuit hash: {:?}", user_circuit.circuit_hash);
            register_cricuit_in_contract(decode_keccak_hex(&user_circuit.circuit_hash)?, &quantum_contract).await?;
            update_user_circuit_data_reduction_status(get_pool().await, &user_circuit.circuit_hash, CircuitReductionStatus::Completed).await?;
        }
        sleep(Duration::from_secs(REGISTER_CIRCUIT_LOOP_DURATION)).await;
    }
}

#[tokio::main]
async fn main() {
    gen_quantum_structs().unwrap();

    dotenv().ok();
    info!(" --- Starting quantum contract --- ");
    let _guard = initialize_logger("quantum_contract.log");
    let _db_pool = get_pool().await;
    let superproof_submission_duration = Duration::from_secs(SUPERPROOF_SUBMISSION_DURATION);

    let task2 = tokio::spawn(async move {
        loop {
            match initialize_superproof_submission_loop(superproof_submission_duration).await {
                Ok(_) => {
                    info!("contract poller exit without any error");
                    info!("Restarting in {} mins...", (SUPERPROOF_SUBMISSION_RETRY/60).to_string());
                    sleep(Duration::from_secs(SUPERPROOF_SUBMISSION_RETRY)).await;
                },
                Err(e) => {
                    error!("contract poller exit with error: {:?}", e.root_cause().to_string());
                    info!("Restarting in {} mins...", (SUPERPROOF_SUBMISSION_RETRY/60).to_string());
                    sleep(Duration::from_secs(SUPERPROOF_SUBMISSION_RETRY)).await;
                },
        }
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