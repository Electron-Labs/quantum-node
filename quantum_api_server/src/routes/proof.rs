use anyhow::Result as AnyhowResult;
use quantum_types::{enums::proving_schemes::ProvingSchemes, types::{config::ConfigData, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof}, gnark_plonk::{GnarkPlonkSolidityProof, GnarkPlonkPis}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof} }};
use quantum_utils::error_line;
use rocket::{get, post, serde::json::Json, State};
use tracing::error;

use crate::{error::error::CustomError, service::proof::{get_proof_data_exec, submit_proof_exec}, types::{auth::AuthToken, submit_proof::{SubmitProofRequest, SubmitProofResponse}, proof_data::ProofDataResponse}};

#[post("/proof", data = "<data>")]
pub async fn submit_proof(_auth_token: AuthToken, data: SubmitProofRequest, config_data: &State<ConfigData>) -> AnyhowResult<Json<SubmitProofResponse>, CustomError>{
    let response: AnyhowResult<SubmitProofResponse>;
    if data.proof_type == ProvingSchemes::GnarkGroth16 {
        response = submit_proof_exec::<GnarkGroth16Proof, GnarkGroth16Pis>(data, config_data).await;
    } else if data.proof_type == ProvingSchemes::Groth16 {
        response = submit_proof_exec::<SnarkJSGroth16Proof, SnarkJSGroth16Pis>(data, config_data).await;
    } else if data.proof_type == ProvingSchemes::Halo2Plonk {
        response = submit_proof_exec::<Halo2PlonkProof, Halo2PlonkPis>(data, config_data).await;
    } else if data.proof_type == ProvingSchemes::GnarkPlonk {
        response = submit_proof_exec::<GnarkPlonkSolidityProof, GnarkPlonkPis>(data, config_data).await;
    } else {
        error!("unsupported proving scheme");
        return Err(CustomError::Internal(error_line!(String::from("/proof Unsupported Proving Scheme"))))
    }
    match response {
        Ok(resp)  => Ok(Json(resp)),
        Err(e) => {
            error!("Error in /proof: {:?}",e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}

#[get("/proof/<proof_hash>")]
pub async fn get_proof_status(_auth_token: AuthToken, proof_hash: String, config_data: &State<ConfigData>) -> AnyhowResult<Json<ProofDataResponse>, CustomError> {
    let response = get_proof_data_exec(proof_hash, config_data).await;
    match response{
        Ok(r) => Ok(Json(r)),
        Err(e) => {
            error!("Error in /proof/<proof_id>: {:?}",e);
            Err(CustomError::Internal(e.root_cause().to_string()))
        }
    }
}