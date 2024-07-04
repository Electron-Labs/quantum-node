use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

use super::gnark_groth16::GnarkGroth16Vkey;

#[derive(Debug)]
pub struct ProofGenerationConfig {
    pub inner_proof_path: String,
    pub inner_vk_path: String,
    pub inner_pis_path: String,
    pub outer_pk_bytes: Vec<u8>,
    pub outer_vk: GnarkGroth16Vkey 
}

#[derive(Debug)]
pub struct InnerProofGenerationConfig <T: Proof, V: Vkey, P: Pis>{
    pub scheme_inner_proof: T,
    pub scheme_inner_vk: V,
    pub scheme_inner_pis: P
}