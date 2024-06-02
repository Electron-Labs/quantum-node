use crate::types::{gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}};

pub struct ReductionCircuitBuildResult {
    pub success: bool,
    pub msg: String,
    pub proving_key_bytes: Vec<u8>, // We will use pk_bytes throughout since its too big and object format has no utility
    pub verification_key: GnarkGroth16Vkey  
}

pub struct GenerateReductionProofResult {
    pub success: bool,
    pub msg: String,
    pub reduced_proof: GnarkGroth16Proof,
    pub reduced_pis: GnarkGroth16Pis
}

pub struct GenerateAggregatedProofResult {
    pub success: bool,
    pub msg: String,
    pub aggregated_proof: GnarkGroth16Proof,
    pub new_root: Vec<u8>,
    pub new_leaves: Vec<u8>
}

pub trait CircuitInteractor {
    // Build reducer circuit when inner circuit is gnark groth16
    fn build_gnark_groth16_circuit(inner_vk: GnarkGroth16Vkey, pis_len: usize) -> ReductionCircuitBuildResult;
    // Build reducer circuit when inner circuit is circom groth16
    fn build_snarkjs_groth16_circuit(inner_vk: SnarkJSGroth16Vkey) -> ReductionCircuitBuildResult;
    // Generate reduction circuit proof corresponding to inner gnark groth16 proof
    fn generate_gnark_groth16_reduced_proof(inner_proof: GnarkGroth16Proof, inner_vk: GnarkGroth16Vkey, inner_pis: GnarkGroth16Pis, outer_vk: GnarkGroth16Vkey, outer_pk_bytes: Vec<u8>)-> GenerateReductionProofResult;
    // Generate reduction circuit proof corresponding to inner snarkjs groth16 proof
    fn generate_snarkjs_groth16_reduced_proof(inner_proof: SnarkJSGroth16Proof, inner_vk: SnarkJSGroth16Vkey, inner_pis: SnarkJSGroth16Pis, outer_vk: GnarkGroth16Vkey, outer_pk_bytes: Vec<u8>)-> GenerateReductionProofResult;
    // Generate Aggregated Proof corresponding to bunch of reduced proofs
    fn generate_aggregated_proof(reduced_proofs: Vec<GnarkGroth16Proof>, reduced_pis: Vec<GnarkGroth16Pis>, reduction_circuit_vkeys: Vec<GnarkGroth16Vkey>, aggregator_circuit_pkey: Vec<u8>, aggregator_circuit_vkey: GnarkGroth16Vkey) -> GenerateAggregatedProofResult;
}