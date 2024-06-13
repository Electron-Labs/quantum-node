#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::{path, str::FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use num_bigint::BigUint;
use quantum_utils::file::{dump_object, read_bytes_from_file, read_file, write_bytes_to_file};
use serde::{Deserialize, Serialize};

use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;
use keccak_hash::keccak;
use crate::{error_line, traits::{pis::Pis, proof::Proof, vkey::Vkey}, types::gnark_groth16::{Fq, Fq2, Fq_2, G1Struct, G2Struct, PedersenCommitmentKey}};

use super::{config::ConfigData, gnark_groth16::GnarkGroth16Vkey};

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
    pub IC: Vec<Vec<String>>
}

impl SnarkJSGroth16Vkey {
    pub fn validate_fq_point(fq: &Vec<String>) -> AnyhowResult<()> {
        if fq.len() != 3 || fq[2] != "1" {
            return Err(anyhow!(error_line!("fq point is not valid")));
        }
        let x = ark_bn254::Fq::from(BigUint::from_str(&fq[0]).unwrap());
        let y = ark_bn254::Fq::from(BigUint::from_str(&fq[1]).unwrap());
        let p = ark_bn254::G1Affine::new_unchecked(x, y);
        let is_valid = p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve();
        if !is_valid {
            return Err(anyhow!(error_line!("fq point is not valid")));
        }
        Ok(())
    }

    pub fn validate_fq2_point(fq2: &Vec<Vec<String>>)  -> AnyhowResult<()>{
        if fq2.len() != 3 || fq2[2].len() != 2 ||  fq2[2][0] != "1" || fq2[2][1] != "0" {
            return Err(anyhow!(error_line!("fq2 point is not valid")));
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
            return Err(anyhow!(error_line!("fq2 point is not valid")));
        }
        Ok(())
    }
}

impl SnarkJSGroth16Vkey {
	pub fn convert_to_gnark_vkey(&self) -> GnarkGroth16Vkey {
		let mut k: Vec<Fq> = Vec::new();
		let ic = self.IC.clone();
		for i in 0..ic.len() {
			let fq = Fq {
				X: ic[i][0].clone(),
				Y: ic[i][1].clone(),
			};
			k.push(fq);
		}
		let gnark_converted_vkey = GnarkGroth16Vkey {
			G1: G1Struct {
					Alpha: Fq {
						X: self.vk_alpha_1[0].clone(),
						Y: self.vk_alpha_1[1].clone(),
					},
					// putting dummy data in Beta
					Beta: Fq {
						X: self.vk_alpha_1[0].clone(),
						Y: self.vk_alpha_1[1].clone(),
					},
					// putting dummy data in Beta
					Delta: Fq {
						X: self.vk_alpha_1[0].clone(),
						Y: self.vk_alpha_1[1].clone(),
					},
					K: k,
				},
			G2: G2Struct {
					Beta: Fq2 {
						X: Fq_2 {
							A0: self.vk_beta_2[0][0].clone(),
							A1: self.vk_beta_2[0][1].clone(),
						},
						Y: Fq_2 {
							A0: self.vk_beta_2[1][0].clone(),
							A1: self.vk_beta_2[1][1].clone(),
						},
					},
					Delta: Fq2 {
						X: Fq_2 {
							A0: self.vk_delta_2[0][0].clone(),
							A1: self.vk_delta_2[0][1].clone(),
						},
						Y: Fq_2 {
							A0: self.vk_delta_2[1][0].clone(),
							A1: self.vk_delta_2[1][1].clone(),
						},
					},
					Gamma: Fq2 {
						X: Fq_2 {
							A0: self.vk_gamma_2[0][0].clone(),
							A1: self.vk_gamma_2[0][1].clone(),
						},
						Y: Fq_2 {
							A0: self.vk_gamma_2[1][0].clone(),
							A1: self.vk_gamma_2[1][1].clone(),
						},
					},
				},
			CommitmentKey: PedersenCommitmentKey {
				G: Fq2 {
					X: Fq_2 {
						A0: String::from("0"),
						A1: String::from("0"),
					},
					Y: Fq_2 {
						A0: String::from("0"),
						A1: String::from("0"),
					},
				},
				GRootSigmaNeg: Fq2 {
					X: Fq_2 {
						A0: String::from("0"),
						A1: String::from("0"),
					},
					Y: Fq_2 {
						A0: String::from("0"),
						A1: String::from("0"),
					},
				},
			},
			PublicAndCommitmentCommitted: vec![],
		};
		gnark_converted_vkey
	}
}

impl Vkey for SnarkJSGroth16Vkey {
	fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
		Ok(buffer)
	}

	fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self>{
		let key: SnarkJSGroth16Vkey = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
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
		let gnark_converted_vkey = self.convert_to_gnark_vkey();

		Ok(gnark_converted_vkey.keccak_hash()?)
	}
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Proof {
    pub pi_a: Vec<String>,
    pub pi_b: Vec<Vec<String>>,
    pub pi_c: Vec<String>,
    pub protocol: String,
    pub curve: String
}

impl Proof for SnarkJSGroth16Proof {
	fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: SnarkJSGroth16Proof = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
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
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Pis(pub Vec<String>);

impl Pis for SnarkJSGroth16Pis {
	fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: SnarkJSGroth16Pis = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
		Ok(key)
	}

	fn dump_pis(&self, path: &str) -> AnyhowResult<()> {
		let pis_bytes = self.serialize_pis()?;
		write_bytes_to_file(&pis_bytes, path)?;
		Ok(())
	}

	fn read_pis(full_path: &str) -> AnyhowResult<Self> {
		let pis_bytes = read_bytes_from_file(full_path)?;
		let snarkjs_pis: SnarkJSGroth16Pis = SnarkJSGroth16Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
		Ok(snarkjs_pis)
	}

	fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
		let mut keccak_ip = Vec::<u8>::new();
		for i in 0..self.0.len() {
			keccak_ip.extend(self.0[i].as_bytes().iter().cloned());
		}
		let hash = keccak(keccak_ip).0;
		Ok(hash)
	}
	
	fn get_data(&self) -> AnyhowResult<Vec<String>> {
			Ok(self.0.clone())
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