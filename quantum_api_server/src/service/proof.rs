use quantum_db::repository::{proof_repository::{get_proof_by_proof_hash, insert_proof}, reduction_circuit_repository::get_reduction_circuit_for_user_circuit, superproof_repository::{get_last_verified_superproof, get_superproof_by_id}, task_repository::create_proof_task, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::{circuit_reduction_status::CircuitReductionStatus, proof_status::ProofStatus, task_status::TaskStatus, task_type::TaskType}, traits::{circuit_interactor::{IMT_Tree, KeccakHashOut}, pis::Pis, proof::Proof}, types::{config::ConfigData, db::superproof, gnark_groth16::GnarkGroth16Pis}};
use quantum_utils::{keccak::{convert_string_to_le_bytes, decode_keccak_hex, encode_keccak_hash}, paths::{get_user_pis_path, get_user_proof_path}};
use rocket::State;
use anyhow::{anyhow, Context, Result as AnyhowResult};
use tracing::info;
use crate::{connection::get_pool, error::error::CustomError, types::{proof_data::ProofDataResponse, protocol_proof::ProtocolProofResponse, submit_proof::{SubmitProofRequest, SubmitProofResponse}}};
use keccak_hash::keccak;

pub async fn submit_proof_exec<T: Proof, F: Pis>(data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<SubmitProofResponse>{
    validate_circuit_data_in_submit_proof_request(&data).await.with_context(|| format!("Cannot validate circuit data in circuit_proof_request in file: {} on line: {}", file!(), line!()))?;
    
    let proof: T = T::deserialize_proof(&mut data.proof.as_slice()).with_context(|| format!("Unable to deserialize proof in file: {} on line: {}", file!(), line!()))?;

    let pis: F = F::deserialize_pis(&mut data.pis.as_slice()).with_context(|| format!("Unable to deserialize pis in file: {} on line: {}", file!(), line!()))?;

    let mut proof_id_ip = Vec::<u8>::new();
    let vkey_hash = decode_keccak_hex(&data.circuit_hash).with_context(|| format!("Invalid keccak hash in file: {} on line: {}", file!(), line!()))?;
    proof_id_ip.extend(vkey_hash.to_vec().iter().cloned());
    let pis_data = pis.get_data().with_context(|| format!("Unable to retrieve pis data in file: {} on line: {}", file!(), line!()))?;
    for i in 0..pis_data.len() {
        let pi = pis_data[i].clone();
        proof_id_ip.extend(convert_string_to_le_bytes(&pi).to_vec().iter().cloned());
    }

    let proof_id_hash = keccak(proof_id_ip).0;

    let proof_id = encode_keccak_hash(&proof_id_hash)?;

    check_if_proof_already_exist(&proof_id).await.with_context(|| format!("Unable to check proof existence in file: {} on line: {}", file!(), line!()))?;

    // Dump proof and pis binaries
    let proof_full_path = get_user_proof_path(&config_data.storage_folder_path, &config_data.proof_path, &data.circuit_hash, &proof_id);
    let pis_full_path = get_user_pis_path(&config_data.storage_folder_path, &config_data.public_inputs_path, &data.circuit_hash, &proof_id);
    proof.dump_proof(&proof_full_path)?;
    pis.dump_pis(&pis_full_path)?;

    insert_proof(get_pool().await, &proof_id, &pis_full_path, &proof_full_path, ProofStatus::Registered, &data.circuit_hash).await.with_context(|| format!("Unable to insert proof in file: {} on line: {}", file!(), line!()))?;
    create_proof_task(get_pool().await, &data.circuit_hash, TaskType::ProofGeneration, TaskStatus::NotPicked, &proof_id).await.with_context(|| format!("Unable to create proof task in file: {} on line: {}", file!(), line!()))?;

    Ok(SubmitProofResponse {
        proof_id
    })
}

pub async fn get_proof_data_exec(proof_id: String, config_data: &ConfigData) -> AnyhowResult<ProofDataResponse> {
    let mut response = ProofDataResponse {
        status: ProofStatus::NotFound.to_string(),
        superproof_id: -1,
        transaction_hash: None,
        verification_contract: config_data.verification_contract_address.clone()
    };
    let proof = get_proof_by_proof_hash(get_pool().await, &proof_id).await;
    if proof.is_err() {
        return Ok(response);
    }

    let proof = proof?;
    response.status = proof.proof_status.to_string();
    if proof.superproof_id.is_some() {
        let superproof_id = proof.superproof_id.unwrap_or(0);
        let superproof = get_superproof_by_id(get_pool().await, superproof_id).await;
        let superproof = match superproof {
            Ok(sp) => Ok(sp),
            Err(e) => {
                info!("err in superproof fetch");
                let error_msg = format!("superproof not found in db: {}", e.to_string());
                Err(anyhow!(CustomError::Internal(error_msg))).with_context(|| format!("{} in file: {} on line: {}", file!(), line!(),e.to_string()))
            }
        };
        let superproof = superproof?;

        response.superproof_id = superproof_id as i64;
        response.transaction_hash = superproof.transaction_hash;
    }

    return Ok(response);
}

async fn validate_circuit_data_in_submit_proof_request(data: &SubmitProofRequest) -> AnyhowResult<()>{
    let circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await;
    let circuit_data = match circuit_data {
        Ok(cd) => Ok(cd),
        Err(_) => {
            info!("circuit has not been registered");
            Err(anyhow!(CustomError::BadRequest("circuit hash not found".to_string()))).with_context(|| format!("Circuit hash not found in file: {} on line: {}", file!(), line!()))
        }
    };

    let circuit_data = circuit_data?;
    if circuit_data.circuit_reduction_status != CircuitReductionStatus::Completed {
        info!("circuit reduction not completed");
        return Err(anyhow!(CustomError::BadRequest("circuit reduction not completed".to_string()))).with_context(|| format!("Circuit Reduction not completed in file: {} on line: {}", file!(), line!()));
    }
    if data.proof_type != circuit_data.proving_scheme {
        info!("prove type is not correct");
        return Err(anyhow!(CustomError::BadRequest("prove type is not correct".to_string()))).with_context(|| format!("Proof Type is not completed in file: {} on line: {}", file!(), line!()));
    }
    
    Ok(())
}

pub async fn check_if_proof_already_exist(proof_id: &str) -> AnyhowResult<()> {
    let proof = get_proof_by_proof_hash(get_pool().await, proof_id).await;
    let is_proof_already_registered = match proof {
        Ok(_) => true,
        Err(_) => false
    };
    if is_proof_already_registered {
        info!("proof already exist");
        return Err(anyhow!(CustomError::BadRequest("proof already exist".to_string())));
    }
    Ok(())
}

pub async fn get_protocol_proof_exec(proof_id: &str) -> AnyhowResult<ProtocolProofResponse> {
    let proof = get_proof_by_proof_hash(get_pool().await, proof_id).await.with_context(|| format!("Unable to retrieve proof from proof hash in file: {} on line: {}", file!(), line!()))?;
    if proof.proof_status != ProofStatus::Verified {
        return Err(anyhow!(CustomError::Internal("proof is not verified".to_string())))
    }
    let reduction_circuit = get_reduction_circuit_for_user_circuit(get_pool().await, &proof.user_circuit_hash).await.with_context(|| format!("Cannot generate reduction circuit for user circuit in file: {} on line: {}", file!(), line!()))?;
    let proof_hash = proof_id;
    let reduction_circuit_hash = reduction_circuit.circuit_id;
    let latest_verififed_superproof = match get_last_verified_superproof(get_pool().await).await.with_context(|| format!("Cannot get last verified superproof in file: {} on line: {}", file!(), line!()))? {
        Some(superproof) => Ok(superproof),
        None => Err(anyhow!(CustomError::Internal("last super proof verified not found".to_string()))),
    }?;
    let leaf_path = latest_verififed_superproof.superproof_leaves_path.unwrap();
    let imt_tree = IMT_Tree::read_tree(&leaf_path)?;
    let proof_hash_bytes = decode_keccak_hex(&proof_hash)?;
    let reduction_hash_bytes = decode_keccak_hex(&reduction_circuit_hash)?;
    let mut keccak_ip = Vec::<u8>::new();
    keccak_ip.extend(reduction_hash_bytes.to_vec().iter().cloned());
    keccak_ip.extend(proof_hash_bytes[0..16].to_vec().iter().cloned());
    keccak_ip.extend([0u8; 16].to_vec().iter().cloned());
    keccak_ip.extend(proof_hash_bytes[16..32].to_vec().iter().cloned());
    keccak_ip.extend([0u8; 16].to_vec().iter().cloned());
    let leaf_val = keccak_hash::keccak(keccak_ip).0;
    let mt_proof = imt_tree.get_imt_proof(KeccakHashOut(leaf_val))?;
    
    let mt_proof_encoded = mt_proof.0.iter().map(|x| encode_keccak_hash(x.as_slice()[0..32].try_into().unwrap()).unwrap()).collect::<Vec<String>>();

    let mut merkle_proof_position: u64 = 0;
    for i in 0..mt_proof.1.len() {
        merkle_proof_position += (mt_proof.1[i] as u64) * 2u64.pow(i as u32);
    }

    let leaf_next_index_str = format!("0x{}", hex::encode(&mt_proof.2.next_idx));

    Ok(ProtocolProofResponse {
        protocol_vkey_hash: proof.user_circuit_hash,
        reduction_vkey_hash: reduction_circuit_hash,
        merkle_proof_position: merkle_proof_position,
        merkle_proof: mt_proof_encoded,
        leaf_next_value: encode_keccak_hash(&mt_proof.2.next_value.0)?,
        leaf_next_index: leaf_next_index_str,
        superproof_root: latest_verififed_superproof.superproof_root.unwrap()
    })
}