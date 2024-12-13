use anyhow::Result as AnyhowResult;
use quantum_db::repository::{protocol::get_protocol_by_auth_token, user_circuit_data_repository::get_user_circuit_data_by_circuit_hash};
use quantum_types::{enums::proving_schemes::ProvingSchemes, traits::{pis::Pis, proof::Proof}, types::{config::ConfigData, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, gnark_plonk::{GnarkPlonkPis, GnarkPlonkSolidityProof, GnarkPlonkVkey}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey}, halo2_poseidon::{Halo2PoseidonPis, Halo2PoseidonProof, Halo2PoseidonVkey}, plonk2::{Plonky2Pis, Plonky2Proof, Plonky2Vkey}, riscs0::{Risc0Pis, Risc0Proof, Risc0Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}, sp1::{Sp1Pis, Sp1Proof, Sp1Vkey} }};
use quantum_utils::error_line;
use rocket::{get, post, serde::json::Json, State};
use tracing::{error, info};

use crate::{connection::get_pool, error::error::CustomError, service::proof::{get_proof_data_exec, submit_proof_exec}, types::{auth::AuthToken, proof_data::ProofDataResponse, submit_proof::{SubmitProofRequest, SubmitProofResponse}}};

#[post("/proof", data = "<data>")]
pub async fn submit_proof(auth_token: AuthToken, mut data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<SubmitProofResponse>, CustomError>{
    // Retrieve protocol data corresponding to the auth token
    let protocol = get_protocol_by_auth_token(get_pool().await, &auth_token.0).await?;
    
    // Check if any such protocol exists
    let protocol = match protocol {
        Some(p) => p,
        None => {
            error!("No protocol against this auth token");
            return Err(CustomError::Internal(error_line!("/register_circuit No protocol against this auth token".to_string())));
        },
    };

    // Retrieve the circuit data corresponding to circuit_hash in the request
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &data.circuit_hash).await?;
    
    // Check if this auth key is allowed to submit proof for this circuit
    if user_circuit_data.protocol_name.to_uppercase() != protocol.protocol_name.to_uppercase() {
        info!("Not authorised to submit proof for this circuit");
        return Err(CustomError::BadRequest(error_line!("Not authorised to submit proof for this circuit".to_string())));
    }

    // Match the proof_type and start executing the submit proof request
    let response: AnyhowResult<SubmitProofResponse> = match data.proof_type {
        ProvingSchemes::GnarkGroth16 => submit_proof_exec::<GnarkGroth16Proof, GnarkGroth16Pis, GnarkGroth16Vkey>(data, config_data).await,
        ProvingSchemes::Groth16 => submit_proof_exec::<SnarkJSGroth16Proof, SnarkJSGroth16Pis, SnarkJSGroth16Vkey>(data, config_data).await,
        ProvingSchemes::GnarkPlonk => submit_proof_exec::<GnarkPlonkSolidityProof, GnarkPlonkPis, GnarkPlonkVkey>(data, config_data).await,
        ProvingSchemes::Halo2Poseidon => submit_proof_exec::<Halo2PoseidonProof, Halo2PoseidonPis, Halo2PoseidonVkey>(data, config_data).await,
        ProvingSchemes::Halo2Plonk => submit_proof_exec::<Halo2PlonkProof, Halo2PlonkPis, Halo2PlonkVkey>(data, config_data).await,
        ProvingSchemes::Plonky2 => {
            let proof = Plonky2Proof::deserialize_proof(&mut data.proof.as_slice())?;
            let pis: Vec<String> = proof.get_pis_strings(&user_circuit_data.vk_path)?;
            let plonk2_pis = Plonky2Pis(pis);
            data.pis = plonk2_pis.serialize_pis()?;
            submit_proof_exec::<Plonky2Proof, Plonky2Pis, Plonky2Vkey>(data, config_data).await
        },
        ProvingSchemes::Sp1 => {
            let proof = Sp1Proof::deserialize_proof(&mut data.proof.as_slice())?;
            let pis_bytes = proof.get_proof_with_public_inputs()?.public_values.to_vec();
            let pis = hex::encode(pis_bytes);
            let sp1_pis = Sp1Pis(vec![pis]);
            data.pis = sp1_pis.serialize_pis()?;
            submit_proof_exec::<Sp1Proof, Sp1Pis, Sp1Vkey>(data, config_data).await
        },
        ProvingSchemes::Risc0 => {
            let proof = Risc0Proof::deserialize_proof(&mut data.proof.as_slice())?;
            let pis_bytes = proof.get_receipt()?.journal.bytes;
            let pis = hex::encode(pis_bytes);
            let risc0_pis = Risc0Pis(vec![pis]);
            data.pis = risc0_pis.serialize_pis()?;
            submit_proof_exec::<Risc0Proof, Risc0Pis, Risc0Vkey>(data, config_data).await
        },
    };
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => {
            error!("Error in /proof: {:?}",e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}

#[get("/proof/<proof_hash>")]
pub async fn get_proof_status(_auth_token: AuthToken, proof_hash: &str, config_data: &State<ConfigData>) -> AnyhowResult<Json<ProofDataResponse>, CustomError> {
    match get_proof_data_exec(proof_hash, config_data).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => {
            error!("Error in /proof/<proof_id>: {:?}",e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}