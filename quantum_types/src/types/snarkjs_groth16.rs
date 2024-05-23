#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::file::{dump_object, read_file};
use serde::{Deserialize, Serialize};

use anyhow::Result as AnyhowResult;

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

use super::config::ConfigData;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Vkey {
	protocol: String,
	curve: String,
    nPublic: u32,
    vk_alpha_1: Vec<String>,
    vk_beta_2: Vec<Vec<String>>,
    vk_gamma_2: Vec<Vec<String>>,
    vk_delta_2: Vec<Vec<String>>,
    vk_alphabeta_12: Vec<Vec<Vec<String>>>,
    IC: Vec<Vec<String>>
}

impl Vkey for SnarkJSGroth16Vkey {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self>{
		let key: SnarkJSGroth16Vkey = BorshDeserialize::deserialize(bytes)?;
		Ok(key)
	}

	fn dump_vk(&self, circuit_hash: &str, config_data: &ConfigData) -> AnyhowResult<String> {
		let vk_path = format!("{}/{}{}", config_data.storage_folder_path, circuit_hash, config_data.user_data_path);
   		let vk_key_full_path = format!("{}/vkey.json", vk_path.as_str() );
    	dump_object(&self, vk_path.as_str(), "vkey.json")?;
		Ok(vk_key_full_path)
	}

	fn read_vk(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_vkey: SnarkJSGroth16Vkey = serde_json::from_str(&json_data)?;
		Ok(gnark_vkey)
	}
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Proof {
    pi_a: Vec<String>,
    pi_b: Vec<Vec<String>>,
    pi_c: Vec<String>,
    protocol: String,
    curve: String
}

impl Proof for SnarkJSGroth16Proof {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: SnarkJSGroth16Proof = BorshDeserialize::deserialize(bytes)?;
		Ok(key)
	}

	fn dump_proof(&self, circuit_hash: &str, config_data: &ConfigData, proof_id: &str) -> AnyhowResult<String> {
		let proof_path = format!("{}/{}{}", config_data.storage_folder_path, circuit_hash, config_data.proof_path);
		let file_name = format!("proof_{}.json", proof_id);
   		let proof_key_full_path = format!("{}/{}", proof_path.as_str(),&file_name );
		// println!("{;]:?}")
    	dump_object(&self, &proof_path, &file_name)?;
		Ok(proof_key_full_path)
	}

	fn read_proof(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_vkey: SnarkJSGroth16Proof = serde_json::from_str(&json_data)?;
		Ok(gnark_vkey)
	}
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Pis(pub Vec<String>);

impl Pis for SnarkJSGroth16Pis {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: SnarkJSGroth16Pis = BorshDeserialize::deserialize(bytes)?;
		Ok(key)
	}

	fn dump_pis(&self, circuit_hash: &str, config_data: &ConfigData, proof_id: &str) -> AnyhowResult<String> {
		let pis_path = format!("{}/{}{}", config_data.storage_folder_path, circuit_hash, config_data.public_inputs_path);
		let file_name = format!("pis_{}.json", proof_id);
   		let pis_key_full_path = format!("{}/{}", pis_path.as_str(), &file_name);
		// println!("{;]:?}")
    	dump_object(&self, &pis_path, &file_name)?;
		Ok(pis_key_full_path)
	}

	fn read_pis(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_pis: SnarkJSGroth16Pis = serde_json::from_str(&json_data)?;
		Ok(gnark_pis)
	}
}



#[cfg(test)]
mod tests {
	use std::fs;
	use borsh::{BorshDeserialize, BorshSerialize};

use super::SnarkJSGroth16Vkey;

    #[test]
    pub fn serde_test() {
        let json_data = fs::read_to_string("./dumps/circom1_vk.json").expect("Failed to read file");
		let snarkjs_vkey: SnarkJSGroth16Vkey = serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

		let mut buffer: Vec<u8> = Vec::new();
		snarkjs_vkey.serialize(&mut buffer).unwrap();
		println!("serialised vkey {:?}", buffer);

		let re_snarkjs_vkey = SnarkJSGroth16Vkey::deserialize(&mut &buffer[..]).unwrap();
		
		assert_eq!(snarkjs_vkey, re_snarkjs_vkey);
    }
}