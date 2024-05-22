#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::file::{dump_object, read_file};
use serde::{Deserialize, Serialize};

use anyhow::Result as AnyhowResult;

use crate::traits::vkey::Vkey;

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

	fn deserialize(bytes: Vec<u8>) -> AnyhowResult<Self>{
		let key: SnarkJSGroth16Vkey = BorshDeserialize::deserialize(&mut bytes.as_slice())?;
		Ok(key)
	}

	fn dump_vk(&self, circuit_hash: &str, storage_path: &str, user_data_path: &str) -> AnyhowResult<String> {
		let vk_path = format!("{}/{}{}", storage_path, circuit_hash, user_data_path);
   		let vk_key_full_path = format!("{}/vk.json", vk_path.as_str() );
    	dump_object(&self, vk_path.as_str(), "vkey.json")?;
		Ok(vk_key_full_path)
	}

	fn read_vk(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_vkey: SnarkJSGroth16Vkey = serde_json::from_str(&json_data)?;
		Ok(gnark_vkey)
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