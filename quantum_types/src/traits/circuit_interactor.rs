use crate::types::{
    config::AMQPConfigData,
    gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey, GnarkVerifier},
    gnark_plonk::{GnarkPlonkPis, GnarkPlonkSolidityProof, GnarkPlonkVkey},
    halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey},
    hash::KeccakHashOut,
    imt::QuantumLeaf,
    snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey},
};
use anyhow::Result as AnyhowResult;

#[derive(Clone, Debug)]
pub struct ReductionCircuitBuildResult {
    pub success: bool,
    pub msg: String,
    pub proving_key_bytes: Vec<u8>, // We will use pk_bytes throughout since its too big and object format has no utility
    pub verification_key: GnarkGroth16Vkey,
}

#[derive(Clone, Debug)]
pub struct GenerateReductionProofResult {
    pub success: bool,
    pub msg: String,
    pub reduced_proof: GnarkGroth16Proof,
    pub reduced_pis: GnarkGroth16Pis,
}

#[derive(Clone, Debug)]
pub struct GenerateAggregatedProofResult {
    pub success: bool,
    pub msg: String,
    pub aggregated_proof: GnarkGroth16Proof,
    pub old_root: Vec<u8>,
    pub new_root: Vec<u8>,
    pub pub_inputs: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct GenerateImtProofResult {
    pub success: bool,
    pub msg: String,
    pub aggregated_proof: GnarkGroth16Proof,
    pub old_root: KeccakHashOut,
    pub new_root: KeccakHashOut,
    pub new_leaves: Vec<QuantumLeaf>,
    pub pub_inputs: Vec<String>,
}

pub trait CircuitInteractorFFI {
    // Build reducer circuit when inner circuit is gnark groth16
    fn build_gnark_groth16_circuit(
        inner_vk: GnarkGroth16Vkey,
        n_pis: usize,
    ) -> ReductionCircuitBuildResult;
    // Build reducer circuit when inner circuit is circom groth16
    fn build_snarkjs_groth16_circuit() -> ReductionCircuitBuildResult;
    // Build reducer circuit when inner circuit is halo2 plonk
    fn build_halo2_plonk_circuit(vk: Halo2PlonkVkey) -> ReductionCircuitBuildResult;
    // Build reducer circuit when inner circuit is gnark plonk
    fn build_gnark_plonk_circuit(
        inner_vk: GnarkPlonkVkey,
        bh: bool, // uses binary hasher in recursion circuit
    ) -> ReductionCircuitBuildResult;
    // Generate reduction circuit proof corresponding to inner gnark groth16 proof
    fn generate_gnark_groth16_reduced_proof(
        inner_proof: GnarkGroth16Proof,
        inner_vk: GnarkGroth16Vkey,
        inner_pis: GnarkGroth16Pis,
        outer_vk: GnarkGroth16Vkey,
        outer_pk_bytes: Vec<u8>,
    ) -> GenerateReductionProofResult;
    // Generate reduction circuit proof corresponding to inner snarkjs groth16 proof
    fn generate_snarkjs_groth16_reduced_proof(
        inner_proof: SnarkJSGroth16Proof,
        inner_vk: SnarkJSGroth16Vkey,
        inner_pis: SnarkJSGroth16Pis,
        outer_vk: GnarkGroth16Vkey,
        outer_pk_bytes: Vec<u8>,
    ) -> GenerateReductionProofResult;
    // Generate reduction circuit proof corresponding to inner halo2 plonk proof
    fn generate_halo2_plonk_reduced_proof(
        inner_pis: Halo2PlonkPis,
        inner_proof: Halo2PlonkProof,
        inner_vk: Halo2PlonkVkey,
        outer_vk: GnarkGroth16Vkey,
        outer_pk_bytes: Vec<u8>,
    ) -> GenerateReductionProofResult;
    // Generate reduction circuit proof corresponding to inner gnark groth16 proof
    fn generate_gnark_plonk_reduced_proof(
        inner_proof: GnarkPlonkSolidityProof,
        inner_vk: GnarkPlonkVkey,
        inner_pis: GnarkPlonkPis,
        outer_vk: GnarkGroth16Vkey,
        outer_pk_bytes: Vec<u8>,
        bh: bool,
    ) -> GenerateReductionProofResult;
}

pub trait CircuitInteractorAMQP {
    // Generate Aggregated Proof corresponding to bunch of reduced proofs
    fn generate_aggregated_proof(
        config_data: &AMQPConfigData,
        batch_size: u64,
        cur_leaves: Vec<QuantumLeaf>,
        reduced_circuit_data_vec: Vec<GnarkVerifier>,
        imt_reduction_circuit_data: GnarkVerifier,
        protocol_vkey_hashes: Vec<Vec<u8>>,
        protocol_pis_hashes: Vec<Vec<u8>>,
        superproof_id: u64,
    ) -> AnyhowResult<GenerateAggregatedProofResult>;

    // Generate IMT corresponding to bunch of reduced proofs
    fn generate_imt_proof(
        config_data: &AMQPConfigData,
        batch_size: u64,
        cur_leaves: Vec<QuantumLeaf>,
        reduced_vkey_hashes: Vec<Vec<u8>>,
        protocol_vkey_hashes: Vec<Vec<u8>>,
        protocol_pis_hashes: Vec<Vec<u8>>,
        superproof_id: u64,
    ) -> AnyhowResult<GenerateImtProofResult>;
}

#[cfg(test)]
mod tests {
    use keccak_hash::keccak;
    use quantum_utils::keccak::{
        convert_string_to_be_bytes, decode_keccak_hex, encode_keccak_hash,
    };

    use crate::{
        traits::{circuit_interactor::KeccakHashOut, pis::Pis, vkey::Vkey},
        types::{
            gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey},
            imt::ImtTree,
        },
    };

    #[test]
    pub fn test() {
        // let tree = ImtTree::read_tree("/home/ubuntu/quantum/quantum-node/storage/superproofs/6/leaves.bin").unwrap();
        // println!("{:?}", tree.leafs[1]);

        // let val: [u8; 32] = tree.leafs[1].next_value.0;
        // let proof_hash = encode_keccak_hash(&val).unwrap();

        let vkey_path = "/home/ubuntu/quantum/quantum-node/storage/0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572/user_data/vkey.bin";
        let pis_path = "/home/ubuntu/quantum/quantum-node/storage/0x413e1cd49f83c319a4a67d03d817d43d1b8c80cdab33b3e7f69a2db71e166572/public_inputs/pis_0x55efc9dce7850312afe32f166cf4b3370a3a1544e81386539a5c4a6a2f700aaa.json";
        let reduction_circuit_vkey_path = "/home/ubuntu/quantum/quantum-node/storage/reduced_circuit/0x0c3df3ef7566705e8fc286738e037911130f89e4c4511686612cfd06da3f3f83/vk.bin";

        let user_vkey = GnarkGroth16Vkey::read_vk(&vkey_path).unwrap();
        let user_pis = GnarkGroth16Pis::read_pis(&pis_path).unwrap();
        let reducn_vkey = GnarkGroth16Vkey::read_vk(&reduction_circuit_vkey_path).unwrap();

        let proof_hash_cached =
            "0x55efc9dce7850312afe32f166cf4b3370a3a1544e81386539a5c4a6a2f700aaa";
        let redn_ckt_hash_cached =
            "0x0c3df3ef7566705e8fc286738e037911130f89e4c4511686612cfd06da3f3f83";

        let proof_hash = decode_keccak_hex(&proof_hash_cached).unwrap();
        let reduction_hash = decode_keccak_hex(&redn_ckt_hash_cached).unwrap();

        let mut keccak_ip = Vec::<u8>::new();
        keccak_ip.extend(reduction_hash.to_vec().iter().cloned());
        keccak_ip.extend(proof_hash[0..16].to_vec().iter().cloned());
        keccak_ip.extend([0u8; 16].to_vec().iter().cloned());
        keccak_ip.extend(proof_hash[16..32].to_vec().iter().cloned());
        keccak_ip.extend([0u8; 16].to_vec().iter().cloned());

        let final_hash = keccak_hash::keccak(keccak_ip).0;

        let tree = ImtTree::read_tree(
            "/home/ubuntu/quantum/quantum-node/storage/superproofs/12/leaves.bin",
        )
        .unwrap();

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
