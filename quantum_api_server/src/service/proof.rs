use crate::{
    connection::get_pool,
    error::error::CustomError,
    types::{
        proof_data::ProofDataResponse,
        protocol_proof::ProtocolProofResponse,
        submit_proof::{SubmitProofRequest, SubmitProofResponse},
    },
};
use aggregation::inputs::compute_leaf_value;
use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::repository::{proof_repository::get_proof_by_proof_hash, superproof_repository::get_superproof_by_id};
use quantum_db::repository::{
    proof_repository::{get_latest_proof_by_circuit_hash, insert_proof},
    task_repository::create_proof_task,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::types::db::proof::Proof as DbProof;
use quantum_types::{
    enums::{
        proof_status::ProofStatus,
        task_status::TaskStatus, task_type::TaskType,
    },
    traits::{pis::Pis, proof::Proof, vkey::Vkey},
    types::config::ConfigData,
};
use quantum_utils::{
    error_line,
    keccak::{decode_keccak_hex, encode_keccak_hash},
    paths::{get_user_pis_path, get_user_proof_path},
};
use rocket::State;
use tracing::info;
use utils::hash::{HashOut, Keccak256Hasher, QuantumHasher};
use tiny_merkle::{proof::Position, MerkleTree};

pub async fn submit_proof_exec<T: Proof, F: Pis, V: Vkey>(
    data: SubmitProofRequest,
    config_data: &State<ConfigData>,
) -> AnyhowResult<SubmitProofResponse> {
    // Checks if proving type is correct and there is no in proocess proof aggregation going on this circuit hash
    validate_circuit_data_in_submit_proof_request(&data).await?;
    // Get `UserCircuitData` corresponding to the circuit hash
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await?;

    // Check if the submitted proof is valid
    let proof: T = T::deserialize_proof(&mut data.proof.as_slice())?;
    proof.validate_proof(&user_circuit_data.vk_path, data.pis.as_slice())?;

    let pis: F = F::deserialize_pis(&mut data.pis.as_slice())?;

    let user_vk = V::read_vk(&user_circuit_data.vk_path)?;

    let proof_id_hash = Keccak256Hasher::combine_hash(&user_vk.keccak_hash()?, &pis.keccak_hash()?);
    let proof_hash = encode_keccak_hash(&proof_id_hash)?;

    // Ensure same proof wasnt submitted before
    match check_if_proof_already_exist(&proof_hash).await {
        Ok(v) => {
            if v {
                return Err(anyhow!(CustomError::Internal(format!(
                    "Proof {:?} already exists", proof_hash
                ))));
            }
        }
        Err(_) => {
            return Err(anyhow!(CustomError::Internal(error_line!(
                "Some issue in check if proof already exists or not".to_string()
            ))));
        }
    }

    // Dump proof and pis binaries
    let proof_full_path = get_user_proof_path(
        &config_data.storage_folder_path,
        &config_data.proof_path,
        &data.circuit_hash,
        &proof_hash,
    );
    let pis_full_path = get_user_pis_path(
        &config_data.storage_folder_path,
        &config_data.public_inputs_path,
        &data.circuit_hash,
        &proof_hash,
    );
    proof.dump_proof(&proof_full_path)?;
    pis.dump_pis(&pis_full_path)?;

    let public_inputs_json_string = serde_json::to_string(&pis.get_data()?)?;

    // Store proof information to the DB
    let db_proof_id = insert_proof(
        get_pool().await,
        &proof_hash,
        &pis_full_path,
        &proof_full_path,
        ProofStatus::Registered,
        &data.circuit_hash,
        &public_inputs_json_string,
    ).await?;

    // Create a proving task
    create_proof_task(
        get_pool().await,
        &data.circuit_hash,
        TaskType::ProofGeneration,
        TaskStatus::NotPicked,
        &proof_hash,
        db_proof_id,
    ).await?;

    Ok(SubmitProofResponse {
        proof_id: proof_hash,
    })
}

pub async fn get_proof_data_exec(
    proof_hash: &str,
    config_data: &ConfigData,
) -> AnyhowResult<ProofDataResponse> {
    // Get verification contract address
    let verification_contract = &config_data.verification_contract_address;

    // Try to fetch proof, return early if not found
    let proof = match get_proof_by_proof_hash(get_pool().await, &proof_hash).await {
        Ok(p) => p,
        Err(_) => return Ok(ProofDataResponse{
            status: ProofStatus::NotFound.to_string(),
            superproof_id: -1,
            transaction_hash: None,
            verification_contract: verification_contract.clone()
        })
    };

    // Early return if no superproof
    let superproof_id = match proof.superproof_id {
        Some(id) => id,
        None => return Ok(ProofDataResponse {
            status: proof.proof_status.to_string(),
            superproof_id: -1,
            transaction_hash: None,
            verification_contract: verification_contract.clone(),
        }),
    };

    // Fetch superproof with minimal error handling
    let superproof = get_superproof_by_id(get_pool().await, superproof_id).await?;

    return Ok(ProofDataResponse {
        status: superproof.status.to_string(),
        superproof_id: superproof_id.try_into()?,
        transaction_hash: superproof.transaction_hash,
        verification_contract: verification_contract.clone()
    });
}

async fn validate_circuit_data_in_submit_proof_request(
    data: &SubmitProofRequest,
) -> AnyhowResult<()> {
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await?;

    if data.proof_type != circuit_data.proving_scheme {
        info!("prove type is not correct");
        return Err(anyhow!(CustomError::BadRequest(error_line!(
            "prove type is not correct".to_string()
        ))));
    }
    validate_on_ongoing_proof_with_same_circuit_hash(&data.circuit_hash).await?;
    Ok(())
}

pub async fn validate_on_ongoing_proof_with_same_circuit_hash(
    circuit_hash: &str,
) -> AnyhowResult<()> {
    // Check if any latest proof exists, if no proof found still return Ok()
    let proof = match get_latest_proof_by_circuit_hash(get_pool().await, circuit_hash).await {
        Ok(p) => p,
        Err(_) => {
            // Still return Ok() if no proof has been aggregated ever before
            return Ok(())
        }
    };

    // If any proof is in progress return Err
    if proof.proof_status == ProofStatus::Registered
        || proof.proof_status == ProofStatus::Reducing
        || proof.proof_status == ProofStatus::Reduced
        || proof.proof_status == ProofStatus::Aggregating
    {
        return Err(anyhow!(CustomError::BadRequest(
            error_line!(format!("Process for a previous proof for this circuit id {:?} hasn't been completed, rejecting anymore proof submission request till then", circuit_hash
        )))));
    }
    Ok(())
}


pub async fn check_if_proof_already_exist(proof_hash: &str) -> AnyhowResult<bool> {
    Ok(get_proof_by_proof_hash(get_pool().await, proof_hash).await.is_ok())
}

pub async fn get_protocol_proof_exec<T: Pis, V: Vkey>(
    proof: &DbProof,
) -> AnyhowResult<ProtocolProofResponse, CustomError> {
    type H = Keccak256Hasher;

    // Extract superproof to which proof belongs
    let superproof = get_superproof_by_id(get_pool().await, proof.superproof_id.ok_or(anyhow!("missing superproof_id"))?).await?;

    // Get leaf value(`target_leaf`) for this user proof
    let circuit_hash = decode_keccak_hex(&proof.user_circuit_hash.clone())?;
    let pis: T = T::read_pis(&proof.pis_path)?;
    let protocol_pis_hash = pis.keccak_hash()?;
    let target_leaf = compute_leaf_value::<Keccak256Hasher>(&circuit_hash, &protocol_pis_hash);

    // Extract all the leaves for this superproof
    let leaves: Vec<HashOut> = read_superproof_leaves(&superproof.r0_leaves_path.ok_or(anyhow!("missing risc0 leaves path"))?)?;

    // Get merkle tree proof
    let mt_proof = get_mt_proof::<H>(&leaves, &target_leaf)?;

    // Merkle proof encoded as hex string
    let mt_proof_encoded = mt_proof
        .0
        .iter()
        .map(|x| encode_keccak_hash(x.as_slice()[0..32].try_into().unwrap()).unwrap())
        .collect::<Vec<String>>();

    // Encode merkle proof positions as a u64
    let merkle_proof_position: u64 = mt_proof.1.iter()
    .enumerate()
    .fold(0, |acc, (i, &pos)| acc | ((pos as u64) << i));

    Ok(ProtocolProofResponse {
        merkle_proof_position,
        merkle_proof: mt_proof_encoded,
    })
}

// Assumes that leaves are stored as [u8;32]'s
pub fn read_superproof_leaves(
    superproof_leaves_path: &str
) -> AnyhowResult<Vec<[u8; 32]>> {
    // Extract dumped leaves
    Ok(bincode::deserialize(&std::fs::read(&superproof_leaves_path)?)?)
}

pub fn get_mt_proof<H: QuantumHasher>(
    leaves: &Vec<HashOut>,
    target_leaf: &HashOut,
) -> AnyhowResult<(Vec<HashOut>, Vec<u8>)> {
    // Check if `target_leaf` exists in the `leaves` set
    if !leaves.iter().any(|leaf| leaf == target_leaf) {
        return Err(anyhow!(error_line!(
            "Target leaf is absent in provided leaves"
        )));
    }

    // Build the merkle tree
    let merkle_leaves: Vec<_> = leaves
    .iter()
    .map(|leaf| H::to_internal_hash(leaf))
    .collect::<Result<_, _>>()?;
    let mtree = MerkleTree::<H>::from_leaves(merkle_leaves, None);
    let mt_proof = mtree.proof(H::to_internal_hash(target_leaf)?).ok_or_else(|| anyhow!("Couldn't find a valid Merkle proof"))?;

    // Extract proof values and directions
    let proof_and_positions = mt_proof.proofs.iter().map(|elm| {
        let position_bit = match elm.position {
            Position::Left => 0,
            Position::Right => 1
        };
        Ok((H::value_from_slice(elm.data.as_ref())?, position_bit))
    }).collect::<Result<Vec<_>, anyhow::Error>>()?;

    let (proof, positions) = proof_and_positions.into_iter().unzip();

    Ok((proof, positions))
}
