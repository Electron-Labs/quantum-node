#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use agg_core::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use gnark_bn254_verifier::{
    converter::fr_from_be_bytes_mod_order,
    structs::{CircomProof, CircomVK, Groth16Proof},
    verify::Groth16VerifyingKey,
};
use num_bigint::BigUint;
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
    keccak::pub_inputs_str_to_fr,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::info;
use utils::{
    hash::{Hasher, KeccakHasher},
    public_inputs_hash, public_inputs_hash_fr,
};
use halo2curves_axiom::bn256::Fr;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Vkey {
    pub protocol: String,
    pub curve: String,
    pub nPublic: u32,
    pub vk_alpha_1: Vec<String>,
    pub vk_beta_2: Vec<Vec<String>>,
    pub vk_gamma_2: Vec<Vec<String>>,
    pub vk_delta_2: Vec<Vec<String>>,
    pub vk_alphabeta_12: Vec<Vec<Vec<String>>>,
    pub IC: Vec<Vec<String>>,
}

impl SnarkJSGroth16Vkey {
    pub fn curve_vk(&self) -> AnyhowResult<Groth16VerifyingKey> {
        Groth16VerifyingKey::from_circom_vk(CircomVK {
            vk_alpha_1: self.vk_alpha_1.clone(),
            vk_beta_2: self.vk_beta_2.clone(),
            vk_gamma_2: self.vk_gamma_2.clone(),
            vk_delta_2: self.vk_delta_2.clone(),
            vk_alphabeta_12: self.vk_alphabeta_12.clone(),
            IC: self.IC.clone(),
        })
    }
}

impl Vkey for SnarkJSGroth16Vkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Vkey =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_vk(&self, path: &str) -> AnyhowResult<()> {
        let vkey_bytes = self.serialize_vkey()?;
        write_bytes_to_file(&vkey_bytes, path)?;
        Ok(())
    }

    fn read_vk(full_path: &str) -> AnyhowResult<Self> {
        let vkey_bytes = read_bytes_from_file(full_path)?;
        let snarkjs_vkey = SnarkJSGroth16Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(snarkjs_vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        self.curve_vk()?;
        info!("Snarkjs vkey validated");
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let curve_vk = self.curve_vk()?;
        let vk_bytes = bincode::serialize(&curve_vk).unwrap();
        Ok(KeccakHasher::hash_out(&vk_bytes))
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        // let gnark_converted_vkey = self.convert_to_gnark_vkey();
        // gnark_converted_vkey.compute_circuit_hash(circuit_verifying_id)
        let pvk_hash = self.keccak_hash()?;

        let circuit_hash =
            compute_combined_vkey_hash::<KeccakHasher>(&pvk_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Proof {
    pub pi_a: Vec<String>,
    pub pi_b: Vec<Vec<String>>,
    pub pi_c: Vec<String>,
    pub protocol: String,
    pub curve: String,
}

impl Proof for SnarkJSGroth16Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Proof =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_proof(&self, path: &str) -> AnyhowResult<()> {
        let proof_bytes = self.serialize_proof()?;
        write_bytes_to_file(&proof_bytes, path)?;
        Ok(())
    }

    fn read_proof(full_path: &str) -> AnyhowResult<Self> {
        let proof_bytes = read_bytes_from_file(full_path)?;
        let snarkjs_proof = SnarkJSGroth16Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(snarkjs_proof)
    }

    fn validate_proof(&self, vkey_path: &str, mut pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vkey = SnarkJSGroth16Vkey::read_vk(vkey_path)?;
        let pis = SnarkJSGroth16Pis::deserialize_pis(&mut pis_bytes)?;
        let mut curve_vk = vkey.curve_vk()?;
        let curve_proof = self.curve_proof()?;
        let public_inputs = pis.to_fr();

        let is_verified = gnark_bn254_verifier::verify::verify_groth16(
            &mut curve_vk,
            &curve_proof,
            &public_inputs.as_slice(),
        )?;
        if !is_verified {
            return Err(anyhow!(error_line!(
                "snarkjs-groth16 proof validation failed"
            )));
        }

        Ok(())
    }
}

impl SnarkJSGroth16Proof {
    pub fn curve_proof(&self) -> AnyhowResult<Groth16Proof> {
        Groth16Proof::from_circom_proof(CircomProof {
            pi_a: self.pi_a.clone(),
            pi_b: self.pi_b.clone(),
            pi_c: self.pi_c.clone(),
            protocol: self.protocol.clone(),
            curve: self.curve.clone(),
        })
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Pis(pub Vec<String>);

impl Pis for SnarkJSGroth16Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Pis =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_pis(&self, path: &str) -> AnyhowResult<()> {
        let pis_bytes = self.serialize_pis()?;
        write_bytes_to_file(&pis_bytes, path)?;
        Ok(())
    }

    fn read_pis(full_path: &str) -> AnyhowResult<Self> {
        let pis_bytes = read_bytes_from_file(full_path)?;
        let snarkjs_pis: SnarkJSGroth16Pis =
            SnarkJSGroth16Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(snarkjs_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        Ok(public_inputs_hash_fr::<KeccakHasher>(&self.to_fr()))
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}

impl SnarkJSGroth16Pis {
    pub fn to_fr(&self) -> Vec<Fr> {
        pub_inputs_str_to_fr(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use std::fs;

    use super::SnarkJSGroth16Vkey;

    #[test]
    pub fn serde_test() {
        let json_data = fs::read_to_string("./dumps/circom1_vk.json").expect("Failed to read file");
        let snarkjs_vkey: SnarkJSGroth16Vkey =
            serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

        let mut buffer: Vec<u8> = Vec::new();
        snarkjs_vkey.serialize(&mut buffer).unwrap();
        println!("serialised vkey {:?}", buffer);

        let re_snarkjs_vkey = SnarkJSGroth16Vkey::deserialize(&mut &buffer[..]).unwrap();

        assert_eq!(snarkjs_vkey, re_snarkjs_vkey);
    }
}
