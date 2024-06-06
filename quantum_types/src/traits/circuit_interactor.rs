use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use quantum_utils::file::{read_bytes_from_file, write_bytes_to_file};
use serde::{Deserialize, Serialize};
use anyhow::Result as AnyhowResult;
use crate::types::{gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}};
use tiny_merkle::{proof::Position, Hasher, MerkleTree};

#[derive(Clone, Debug)]
pub struct KeccakHasher;

impl tiny_merkle::Hasher for KeccakHasher {
    type Hash = [u8; 32];

    fn hash(value: &[u8]) -> Self::Hash {
        keccak(value).0
    }
}

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

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
pub struct KeccakHashOut (pub [u8; 32]);

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
pub struct QuantumLeaf {
    pub value: KeccakHashOut,
    pub next_value: KeccakHashOut,
    pub next_idx: [u8; 8]
}

impl QuantumLeaf {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.value.clone().0);
        serialized.extend(self.next_value.clone().0);
        serialized.extend(self.next_idx.clone());
        serialized
    }
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

    pub fn get_mtree(&self) -> MerkleTree<KeccakHasher> {
        let leaves_structs = self.leafs.clone();
        let leaves = leaves_structs
                                        .iter()
                                        .map(|leaf_struct| keccak(&leaf_struct.serialize()).0 )
                                        .collect::<Vec<[u8; 32]>>();
        MerkleTree::<KeccakHasher>::from_leaves(leaves, None)
    }

    pub fn get_imt_proof(&self, leaf_val: KeccakHashOut) -> AnyhowResult<(Vec<Vec<u8>>, Vec<u8>)>{
        let leafs = self.leafs.clone();
        let mut leaf_asked: Option<QuantumLeaf> = None;
        for leaf in leafs {
            if leaf.value == leaf_val {
                leaf_asked = Some(leaf.clone());
                break;
            }
        }
        if leaf_asked.is_none() {
            return Err(anyhow::Error::msg("Couldnt find a value in leaves"));
        }
        let leaf = leaf_asked.unwrap();
        let mtree = self.get_mtree();
        let imt_proof = mtree.proof(keccak(&leaf.serialize()).0);
        if imt_proof.is_none() {
            return Err(anyhow::Error::msg("Couldnt find a valid merkle proof"));
        }
        let mut proof = Vec::<Vec<u8>>::new();
        let mut proof_helper = Vec::<u8>::new();
        proof.push(leaf.next_value.0.to_vec());
        proof.push(leaf.next_idx.to_vec());

        imt_proof.unwrap().proofs.iter().for_each(|elm| {
            proof.push(elm.data.to_vec());
            let posn = &elm.position;
            match posn {
                Position::Left => proof_helper.push(0),
                Position::Right => proof_helper.push(1),
            }
        });

        // return proof = ([next_leaf_val, next_idx, merkle_proof ...], merkle_proof_helper)
        Ok((proof, proof_helper))
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