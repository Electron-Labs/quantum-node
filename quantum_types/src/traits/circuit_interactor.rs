use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::file::{read_bytes_from_file, write_bytes_to_file};
use serde::{Deserialize, Serialize};
use anyhow::Result as AnyhowResult;
use crate::types::{gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}};

#[derive(Clone, Debug)]
pub struct ReductionCircuitBuildResult {
    pub success: bool,
    pub msg: String,
    pub proving_key_bytes: Vec<u8>, // We will use pk_bytes throughout since its too big and object format has no utility
    pub verification_key: GnarkGroth16Vkey  
}

#[derive(Clone, Debug)]
pub struct GenerateReductionProofResult {
    pub success: bool,
    pub msg: String,
    pub reduced_proof: GnarkGroth16Proof,
    pub reduced_pis: GnarkGroth16Pis
}

#[derive(Clone, Debug)]
pub struct GenerateAggregatedProofResult {
    pub success: bool,
    pub msg: String,
    pub aggregated_proof: GnarkGroth16Proof,
    pub new_root: KeccakHashOut,
    pub new_leaves: Vec<QuantumLeaf>
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct KeccakHashOut (pub [u8; 32]);

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct QuantumLeaf {
    pub value: KeccakHashOut,
    pub next_value: KeccakHashOut,
    pub next_idx: [u8; 8]
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct IMT_Tree {
    pub leafs: Vec<QuantumLeaf>
}

impl IMT_Tree {
    pub fn serialise_imt_tree(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
    }

    pub fn deserialise_imt_tree(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let imt_tree: IMT_Tree = BorshDeserialize::deserialize(bytes)?;
        Ok(imt_tree)
    }

    pub fn dump_tree(&self, path: &str) -> AnyhowResult<()> {
        let imt_bytes = self.serialise_imt_tree()?;
        write_bytes_to_file(&imt_bytes, path)?;
        Ok(())
    }

    pub fn read_tree(path: &str) -> AnyhowResult<Self> {
        let imt_bytes = read_bytes_from_file(path)?;
        let imt_tree = IMT_Tree::deserialise_imt_tree(&mut imt_bytes.as_slice())?;
        Ok(imt_tree)
    }
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
    fn generate_aggregated_proof(
        reduced_proofs: Vec<GnarkGroth16Proof>, 
        reduced_pis: Vec<GnarkGroth16Pis>, 
        reduction_circuit_vkeys: Vec<GnarkGroth16Vkey>, 
        old_root: KeccakHashOut,
        old_leaves: Vec<QuantumLeaf>,
        aggregator_circuit_pkey: Vec<u8>, 
        aggregator_circuit_vkey: GnarkGroth16Vkey
    ) -> GenerateAggregatedProofResult;
}