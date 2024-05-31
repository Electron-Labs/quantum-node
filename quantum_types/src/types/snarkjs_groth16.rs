#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use num_bigint::BigUint;
use quantum_utils::file::{dump_object, read_file};
use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;

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

impl SnarkJSGroth16Vkey {
    pub fn validate_fq_point(fq: &Vec<String>) -> AnyhowResult<()> {
        if fq.len() != 3 || fq[2] != "1" {
            return Err(anyhow!("not valid"));
        }
        let x = ark_bn254::Fq::from(BigUint::from_str(&fq[0]).unwrap());
        let y = ark_bn254::Fq::from(BigUint::from_str(&fq[1]).unwrap());
        let p = ark_bn254::G1Affine::new_unchecked(x, y);
        let is_valid = p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve();
        if !is_valid {
            return Err(anyhow!("not valid"));
        }
        Ok(())
    }

    pub fn validate_fq2_point(fq2: &Vec<Vec<String>>)  -> AnyhowResult<()>{
        if fq2.len() != 3 || fq2[2].len() != 2 ||  fq2[2][0] != "1" || fq2[2][1] != "0" {
            return Err(anyhow!("not valid"));
        }
        let x1 = ark_bn254::Fq::from(BigUint::from_str(&fq2[0][0])?);
        let x2 = ark_bn254::Fq::from(BigUint::from_str(&fq2[0][1])?);

        let x = ark_bn254::Fq2::new(x1, x2);

        let y1 = ark_bn254::Fq::from(BigUint::from_str(&fq2[1][0])?);
        let y2 = ark_bn254::Fq::from(BigUint::from_str(&fq2[1][1])?);
        let y = ark_bn254::Fq2::new(y1, y2);
        let p = ark_bn254::G2Affine::new_unchecked(x, y);
        let is_valid = p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve();
        if !is_valid {
            return Err(anyhow!("not valid"));
        }
        Ok(())
    }
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
    
    fn validate(&self, num_public_inputs: u8) -> AnyhowResult<()> {
        if self.IC.len() as u8 != num_public_inputs+1 {
            return Err(anyhow!("not valid"));
        }
        SnarkJSGroth16Vkey::validate_fq_point(&self.vk_alpha_1)?;
        for ic in &self.IC {
            SnarkJSGroth16Vkey::validate_fq_point(ic)?;
        }

        SnarkJSGroth16Vkey::validate_fq2_point(&self.vk_beta_2)?;
        SnarkJSGroth16Vkey::validate_fq2_point(&self.vk_gamma_2)?;
        SnarkJSGroth16Vkey::validate_fq2_point(&self.vk_delta_2)?;
        info!("vkey validated");
        Ok(())
    }
	
	fn keccak_hash(&self) -> AnyhowResult<[u8;32]> {
		todo!()
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
    	dump_object(&self, &pis_path, &file_name)?;
		Ok(pis_key_full_path)
	}

	fn read_pis(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_pis: SnarkJSGroth16Pis = serde_json::from_str(&json_data)?;
		Ok(gnark_pis)
	}
	
	fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
			todo!()
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