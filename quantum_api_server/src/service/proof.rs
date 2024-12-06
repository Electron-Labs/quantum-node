use crate::{
    connection::get_pool,
    error::error::CustomError,
    types::{
        proof_data::ProofDataResponse,
        protocol_proof::ProtocolProofResponse,
        submit_proof::{SubmitProofRequest, SubmitProofResponse},
    },
};
use agg_core::inputs::compute_leaf_value;
use anyhow::{anyhow, Result as AnyhowResult};
use mt_core::tree::get_merkle_tree;
use quantum_db::repository::{proof_repository::get_proof_by_proof_hash, protocol::get_protocol_by_protocol_name, superproof_repository::get_superproof_by_id};
use quantum_db::repository::{
    proof_repository::{get_latest_proof_by_circuit_hash, insert_proof},
    task_repository::create_proof_task,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{enums::proving_schemes::ProvingSchemes, types::db::proof::Proof as DbProof};
use quantum_types::{
    enums::{
        circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus,
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
use tracing::{error, info};
use utils::hash::{Hasher, KeccakHasher};
use tiny_merkle::proof::Position;

pub async fn submit_proof_exec<T: Proof, F: Pis, V: Vkey>(
    data: SubmitProofRequest,
    config_data: &State<ConfigData>,
) -> AnyhowResult<SubmitProofResponse> {
    validate_circuit_data_in_submit_proof_request(&data).await?;

    let proof: T = T::deserialize_proof(&mut data.proof.as_slice())?;
    let pis: F = F::deserialize_pis(&mut data.pis.as_slice())?;

    let user_circuit_data =
        get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await?;
    let user_vk = V::read_vk(&user_circuit_data.vk_path)?;

    let proof_id_hash = KeccakHasher::combine_hash(&user_vk.keccak_hash()?, &pis.keccak_hash()?);
    let proof_hash = encode_keccak_hash(&proof_id_hash)?;

    proof.validate_proof(&user_circuit_data.vk_path, data.pis.as_slice())?;
    info!("proof validated");
    check_if_proof_already_exist(&proof_hash, &data.circuit_hash).await?;

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

    let public_inputs_json_string = serde_json::to_string(&pis.get_data()?).unwrap();
    let proof_id = insert_proof(
        get_pool().await,
        &proof_hash,
        &pis_full_path,
        &proof_full_path,
        if data.proof_type==ProvingSchemes::Sp1 {ProofStatus::Reduced} else {ProofStatus::Registered},
        &data.circuit_hash,
        &public_inputs_json_string,
    )
    .await?;
    if data.proof_type != ProvingSchemes::Sp1 {
        create_proof_task(
            get_pool().await,
            &data.circuit_hash,
            TaskType::ProofGeneration,
            TaskStatus::NotPicked,
            &proof_hash,
            proof_id,
        )
        .await?;
    }

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
    let circuit_data =
        get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await;
    let circuit_data = match circuit_data {
        Ok(cd) => Ok(cd),
        Err(e) => {
            info!("circuit has not been registered");
            Err(anyhow!(CustomError::BadRequest(error_line!(format!(
                "circuit hash not found. {}",
                e.root_cause().to_string()
            )))))
        }
    };

    let circuit_data = circuit_data?;
    if circuit_data.circuit_reduction_status != CircuitReductionStatus::Completed {
        info!("circuit reduction not completed");
        return Err(anyhow!(CustomError::BadRequest(error_line!(
            "circuit reduction not completed".to_string()
        ))));
    }

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
    let proof = match get_latest_proof_by_circuit_hash(get_pool().await, circuit_hash).await {
        Ok(p) => Ok(p),
        Err(e) => {
            error!(
                "error in finding the last proof for circuit hash {:?}: {:?}",
                circuit_hash,
                error_line!(e)
            );
            Err(e)
        }
    };

    if proof.is_err() {
        return Ok(());
    }

    let proof = proof?;
    if proof.proof_status == ProofStatus::Registered
        || proof.proof_status == ProofStatus::Reducing
        || proof.proof_status == ProofStatus::Reduced
    {
        return Err(anyhow!(CustomError::BadRequest(error_line!(format!("last proof for circuit id {:?} hasn't been verified, rejecting proof submission request", circuit_hash)))));
    }
    Ok(())
}

// TODO: need to change
pub async fn check_if_proof_already_exist(
    proof_hash: &str,
    _circuit_hash: &str,
) -> AnyhowResult<()> {
    let proof = get_proof_by_proof_hash(get_pool().await, proof_hash).await;
    if proof.is_ok() {
        let user_circuit = get_user_circuit_data_by_circuit_hash(
            get_pool().await,
            proof?.user_circuit_hash.as_str(),
        )
        .await?;
        let protocol_name = user_circuit.protocol_name;

        let protocol = get_protocol_by_protocol_name(get_pool().await, &protocol_name).await?;
        if protocol.is_proof_repeat_allowed == 0 {
            info!("proof already exist");
            return Err(anyhow!(CustomError::BadRequest(error_line!(
                "proof already exist".to_string()
            ))));
        }
        // else {
        //     let user_circuit_hash = get_user_
        //     let proof = get_proof_by_proof_hash_within_limits(get_pool().await, proof_hash, circuit_hash, 51).await;
        //     if proof.is_ok() {
        //         info!("repeat proof present in the latest 51 proofs");
        //         return Err(anyhow!(CustomError::BadRequest(error_line!("repeat proof present in the latest 51 proofs".to_string()))));
        //     }
        // }
    }
    Ok(())
}

pub async fn get_protocol_proof_exec<T: Pis, V: Vkey>(
    proof: &DbProof,
) -> AnyhowResult<ProtocolProofResponse, CustomError> {
    type H = KeccakHasher;

    let circuit_hash = decode_keccak_hex(&proof.user_circuit_hash.clone())?;
    let user_circuit_data =
        get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash).await?;
    let pis: T = T::read_pis(&proof.pis_path)?;
    let protocol_pis_hash = pis.keccak_hash()?;
    let superproof = get_superproof_by_id(get_pool().await, proof.superproof_id.ok_or(anyhow!("missing superproof_id"))?).await?;

    let leaves: Vec<[u8; 32]>;
    let last_proof_elm: [u8; 32];
    let last_proof_elm_position: u8;
    match user_circuit_data.proving_scheme {
        ProvingSchemes::Sp1 => {
            leaves = read_superproof_leaves::<H>(&superproof.sp1_leaves_path.ok_or(anyhow!("missing sp1 leaves path"))?)?;
            last_proof_elm = decode_keccak_hex(&superproof.r0_root.ok_or(anyhow!("missing r0_root"))?)?;
            last_proof_elm_position = 0;
        },
        _ => {
            leaves = read_superproof_leaves::<H>(&superproof.r0_leaves_path.ok_or(anyhow!("missing risc0 leaves path"))?)?;
            last_proof_elm = decode_keccak_hex(&superproof.sp1_root.ok_or(anyhow!("missing r0_root"))?)?;
            last_proof_elm_position = 1;
        }
    }

    let target_leaf = compute_leaf_value::<KeccakHasher>(&circuit_hash, &protocol_pis_hash);
    let mut mt_proof = get_imt_proof::<H>(leaves, target_leaf)?;

    // append last proof elm
    mt_proof.0.push(last_proof_elm);
    mt_proof.1.push(last_proof_elm_position);

    let mt_proof_encoded = mt_proof
        .0
        .iter()
        .map(|x| encode_keccak_hash(x.as_slice()[0..32].try_into().unwrap()).unwrap())
        .collect::<Vec<String>>();

    let mut merkle_proof_position: u64 = 0;
    for i in 0..mt_proof.1.len() {
        merkle_proof_position += (mt_proof.1[i] as u64) * 2u64.pow(i as u32);
    }

    Ok(ProtocolProofResponse {
        merkle_proof_position,
        merkle_proof: mt_proof_encoded,
    })
}

pub fn read_superproof_leaves<H: Hasher>(
    superproof_leaves_path: &str
) -> AnyhowResult<Vec<H::HashOut>> {
    let leaves: Vec<[u8; 32]> = bincode::deserialize(&std::fs::read(&superproof_leaves_path)?)?;

    let mut leaves_hash_type =  Vec::with_capacity(leaves.len());
    for i in  0..leaves.len() {
        leaves_hash_type.push(H::value_from_slice(leaves[i].clone().as_slice())?);
    }

    Ok(leaves_hash_type)
}

pub fn get_imt_proof<H: Hasher>(
    leaves: Vec<H::HashOut>,
    target_leaf: H::HashOut,
) -> AnyhowResult<(Vec<H::Hash>, Vec<u8>)> {
    let mut leaf_asked: Option<H::HashOut> = None;
    for leaf in leaves.clone() {
        if leaf.as_ref() == target_leaf.as_ref() {
            leaf_asked = Some(target_leaf.clone());
            break;
        }
    }
    if leaf_asked.is_none() {
        return Err(anyhow!(error_line!(
            "Target leaf is absent in provided leaves"
        )));
    }
    let leaf = leaf_asked.unwrap();
    let mtree = get_merkle_tree::<H>(leaves)?;
    let imt_proof = mtree.proof(H::to_internal_hash(leaf)?);
    if imt_proof.is_none() {
        return Err(anyhow::Error::msg("Couldn't find a valid merkle proof"));
    }
    let mut proof = Vec::<H::Hash>::new();
    let mut proof_helper = Vec::<u8>::new();

    imt_proof.unwrap().proofs.iter().for_each(|elm| {
        proof.push(elm.data.clone());
        let posn = &elm.position;
        match posn {
            Position::Left => proof_helper.push(0),
            Position::Right => proof_helper.push(1),
        }
    });

    Ok((proof, proof_helper))
}
