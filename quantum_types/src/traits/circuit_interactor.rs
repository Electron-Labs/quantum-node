use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use quantum_utils::{file::{read_bytes_from_file, write_bytes_to_file}, keccak::encode_keccak_hash};
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

    pub fn get_imt_proof(&self, leaf_val: KeccakHashOut) -> AnyhowResult<(Vec<Vec<u8>>, Vec<u8>, QuantumLeaf)>{
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

        imt_proof.unwrap().proofs.iter().for_each(|elm| {
            proof.push(elm.data.to_vec());
            let posn = &elm.position;
            match posn {
                Position::Left => proof_helper.push(0),
                Position::Right => proof_helper.push(1),
            }
        });

        // return proof = ([next_leaf_val, next_idx, merkle_proof ...], merkle_proof_helper)
        Ok((proof, proof_helper, leaf))
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

#[cfg(test)]
mod tests {
    use keccak_hash::keccak;
    use quantum_utils::keccak::{convert_string_to_le_bytes, decode_keccak_hex, encode_keccak_hash};

    use crate::{traits::{circuit_interactor::KeccakHashOut, pis::Pis, vkey::Vkey}, types::gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey}};

    use super::IMT_Tree;

    #[test]
    pub fn test() {
        // let tree = IMT_Tree::read_tree("/home/ubuntu/quantum/quantum-node/storage/superproofs/6/leaves.bin").unwrap();
        // println!("{:?}", tree.leafs[1]);

        // let val: [u8; 32] = tree.leafs[1].next_value.0;
        // let proof_hash = encode_keccak_hash(&val).unwrap();

        let vkey_path = "/home/ubuntu/quantum/quantum-node/storage/0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572/user_data/vkey.bin";
        let pis_path = "/home/ubuntu/quantum/quantum-node/storage/0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572/public_inputs/pis_0x55efc9dce7850312afe32f166cf4b3370a3a1544e81386539a5c4a6a2f700aaa.json";
        let reduction_circuit_vkey_path = "/home/ubuntu/quantum/quantum-node/storage/reduced_circuit/0x0c3df3ef7566705e8fc286738e037911130f89e4c4511686612cfd06da3f3f83/vk.bin";

        let user_vkey = GnarkGroth16Vkey::read_vk(&vkey_path).unwrap();
        let user_pis = GnarkGroth16Pis::read_pis(&pis_path).unwrap();
        let reducn_vkey = GnarkGroth16Vkey::read_vk(&reduction_circuit_vkey_path).unwrap();

        let proof_hash_cached = "0x55efc9dce7850312afe32f166cf4b3370a3a1544e81386539a5c4a6a2f700aaa";
        let redn_ckt_hash_cached = "0x0c3df3ef7566705e8fc286738e037911130f89e4c4511686612cfd06da3f3f83";


        let proof_hash = decode_keccak_hex(&proof_hash_cached).unwrap();
        let reduction_hash = decode_keccak_hex(&redn_ckt_hash_cached).unwrap();

        let mut keccak_ip = Vec::<u8>::new();
        keccak_ip.extend(reduction_hash.to_vec().iter().cloned());
        keccak_ip.extend(proof_hash[0..16].to_vec().iter().cloned());
        keccak_ip.extend([0u8; 16].to_vec().iter().cloned());
        keccak_ip.extend(proof_hash[16..32].to_vec().iter().cloned());
        keccak_ip.extend([0u8; 16].to_vec().iter().cloned());


        let final_hash =  keccak_hash::keccak(keccak_ip).0;

        let tree = IMT_Tree::read_tree("/home/ubuntu/quantum/quantum-node/storage/superproofs/12/leaves.bin").unwrap();

        let proof = tree.get_imt_proof(KeccakHashOut(final_hash)).unwrap();
        // let mut keccak_ip = Vec::<u8>::new();

        // let vk_hash = user_vkey.keccak_hash().unwrap();

        // keccak_ip.extend(vk_hash.to_vec().iter().cloned());

        // for i in 0..user_pis.0.len() {
        //     let pi = user_pis.0[i].clone();
        //     keccak_ip.extend(convert_string_to_le_bytes(&pi).to_vec().iter().cloned());
        // }

        // let proof_hash =  keccak_hash::keccak(keccak_ip).0;
        // let proof_hash_hex = encode_keccak_hash(&proof_hash).unwrap();
        // println!("proof hash {:?}", proof_hash_hex);


        // let mut keccak_ip_2 = Vec::<u8>::new();
        // let redn_ckt_hash = reducn_vkey.keccak_hash().unwrap();
        // let redn_circuit_hex = encode_keccak_hash(&redn_ckt_hash).unwrap();

        // println!("redn_ckt_hash {:?}", redn_circuit_hex);

        // let mut p1 = proof_hash[0..16].to_vec();
        // p1.extend(vec![0u8; 16]);

        // let mut p2 = proof_hash[16..32].to_vec();
        // p2.extend(vec![0u8; 16]);

        // keccak_ip_2.extend(redn_ckt_hash.to_vec().iter().cloned());
        // keccak_ip_2.extend(p1.to_vec().iter().cloned());
        // keccak_ip_2.extend(p2.to_vec().iter().cloned());


    }
}