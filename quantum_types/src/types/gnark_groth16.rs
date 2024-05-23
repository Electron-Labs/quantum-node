#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use borsh::{BorshSerialize, BorshDeserialize};
use quantum_utils::file::{dump_object, read_file};
use serde::{Serialize, Deserialize};
use anyhow::Result as AnyhowResult;

use crate::traits::vkey::Vkey;

use super::config::ConfigData;
/*
type VerifyingKey struct {
	// [α]₁, [Kvk]₁
	G1 struct {
		Alpha       curve.G1Affine
		Beta, Delta curve.G1Affine   // unused, here for compatibility purposes
		K           []curve.G1Affine // The indexes correspond to the public wires
	}

	// [β]₂, [δ]₂, [γ]₂,
	// -[δ]₂, -[γ]₂: see proof.Verify() for more details
	G2 struct {
		Beta, Delta, Gamma curve.G2Affine
		// contains filtered or unexported fields
	}

	CommitmentKey                pedersen.VerifyingKey
	PublicAndCommitmentCommitted [][]int // indexes of public/commitment committed variables
	// contains filtered or unexported fields
}
 */
// We will represent 1 Fr Element by String
#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq {
    X: String, // Since we dont wanna do any field operations on this serve, String should work
    Y: String
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq_2{
	A0 : String,
	A1 : String
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq2 {
    X: Fq_2,
    Y: Fq_2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G1Struct {
    Alpha: Fq,
    Beta: Fq,
    Delta: Fq,
    K: Vec<Fq>
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G2Struct {
    Beta: Fq2,
    Delta: Fq2,
    Gamma: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct PedersenCommitmentKey {
	G: Fq2,
	GRootSigmaNeg: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Vkey {
	G1: G1Struct,
	G2: G2Struct,
	CommitmentKey: PedersenCommitmentKey,
	// We wont support gnark proofs which have PublicAndCommitmentCommitted non-empty
	PublicAndCommitmentCommitted: Vec<Vec<u32>>
}

impl Vkey for GnarkGroth16Vkey {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: Vec<u8>) -> AnyhowResult<Self>{
		let key: GnarkGroth16Vkey = BorshDeserialize::deserialize(&mut bytes.as_slice())?;
		Ok(key)
	}

	fn dump_vk(&self, circuit_hash: &str, config_data: &ConfigData) -> AnyhowResult<String> {
		let vk_path = format!("{}/{}{}", config_data.storage_folder_path, circuit_hash, config_data.user_data_path);
   		let vk_key_full_path = format!("{}/vkey.json", vk_path.as_str() );
		// println!("{;]:?}")
    	dump_object(&self, vk_path.as_str(), "vkey.json")?;
		Ok(vk_key_full_path)
	}

	fn read_vk(full_path: &str) -> AnyhowResult<Self> {
		let json_data = read_file(full_path)?;
		let gnark_vkey: GnarkGroth16Vkey = serde_json::from_str(&json_data)?;
		Ok(gnark_vkey)
	}
}


#[cfg(test)]
mod tests {
	use std::fs;
	use borsh::{BorshDeserialize, BorshSerialize};
	use super::GnarkGroth16Vkey;

	#[test]
	pub fn serde_test() {
		// Read JSON -> Get Struct -> Borsh Serialise -> Borsh Deserialise -> match
		let json_data = fs::read_to_string("./dumps/gnark_vkey.json").expect("Failed to read file");
		let gnark_vkey: GnarkGroth16Vkey = serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

		let mut buffer: Vec<u8> = Vec::new();
		gnark_vkey.serialize(&mut buffer).unwrap();
		println!("serialised vkey {:?}", buffer);

		let re_gnark_vkey = GnarkGroth16Vkey::deserialize(&mut &buffer[..]).unwrap();
		
		assert_eq!(gnark_vkey, re_gnark_vkey);

		println!("{:?}", re_gnark_vkey);
	}
}