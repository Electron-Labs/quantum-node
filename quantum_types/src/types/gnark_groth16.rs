#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;

use ark_bn254::g1::Config;
use ark_ec::short_weierstrass::Affine;
use borsh::{BorshSerialize, BorshDeserialize};
use num_bigint::BigUint;
use quantum_utils::file::{dump_object, read_file};
use serde::{Serialize, Deserialize};
use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;
use keccak_hash::keccak;

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

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
    pub X: String, // Since we dont wanna do any field operations on this serve, String should work
    pub Y: String
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq_2{
	pub A0 : String,
	pub A1 : String
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq2 {
    pub X: Fq_2,
    pub Y: Fq_2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G1Struct {
    pub Alpha: Fq,
    pub Beta: Fq,
    pub Delta: Fq,
    pub K: Vec<Fq>
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G2Struct {
    pub Beta: Fq2,
    pub Delta: Fq2,
    pub Gamma: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct PedersenCommitmentKey {
	pub G: Fq2,
	pub GRootSigmaNeg: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Vkey {
	pub G1: G1Struct,
	pub G2: G2Struct,
	pub CommitmentKey: PedersenCommitmentKey,
	// We wont support gnark proofs which have PublicAndCommitmentCommitted non-empty
	pub PublicAndCommitmentCommitted: Vec<String>
}

impl GnarkGroth16Vkey {
	pub fn validate_fq_point(fq: &Fq) -> AnyhowResult<()>{
		let x = ark_bn254::Fq::from(BigUint::from_str(&fq.X)?);
		let y = ark_bn254::Fq::from(BigUint::from_str(&fq.Y)?);
		let p = ark_bn254::G1Affine::new_unchecked(x, y);
		let is_valid = GnarkGroth16Vkey::check_if_g1_point_is_valid(&p);
		if !is_valid {
			info!("fq point not valid");
			return Err(anyhow!("not valid point"));
		}
		Ok(())
	}

	pub fn validate_fq2_points(fq2: &Fq2) -> AnyhowResult<()>{
		let x1 = ark_bn254::Fq::from(BigUint::from_str(&fq2.X.A0)?);
		let x2 = ark_bn254::Fq::from(BigUint::from_str(&fq2.X.A1)?);
		let x = ark_bn254::Fq2::new(x1, x2);

		let y1 = ark_bn254::Fq::from(BigUint::from_str(&fq2.Y.A0)?);
		let y2 = ark_bn254::Fq::from(BigUint::from_str(&fq2.Y.A1)?);
		let y = ark_bn254::Fq2::new(y1, y2);

		let p = ark_bn254::G2Affine::new(x, y);
		let is_valid = GnarkGroth16Vkey::check_if_g2_point_is_valid(&p);
		if !is_valid {
			info!("fq2 point not valid");
			return Err(anyhow!("not valid point"));
		}
		Ok(())
	}

	pub fn check_if_g1_point_is_valid(p: &Affine<Config>) -> bool {
		return p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve()
	}

	pub fn check_if_g2_point_is_valid(p: &Affine<ark_bn254::g2::Config>) -> bool {
		return p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve()
	}
}

impl Vkey for GnarkGroth16Vkey {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self>{
		let key: GnarkGroth16Vkey = BorshDeserialize::deserialize(bytes)?;
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
		let gnark_vkey: GnarkGroth16Vkey = serde_json::from_str(&json_data)?;
		Ok(gnark_vkey)
	}

	fn validate(&self, num_public_inputs: u8) -> AnyhowResult<()> {
		GnarkGroth16Vkey::validate_fq_point(&self.G1.Alpha)?;
		for point in &self.G1.K {
			GnarkGroth16Vkey::validate_fq_point(point)?;
		}
		GnarkGroth16Vkey::validate_fq2_points(&self.G2.Beta)?;
		GnarkGroth16Vkey::validate_fq2_points(&self.G2.Delta)?;
		GnarkGroth16Vkey::validate_fq2_points(&self.G2.Gamma)?;
		GnarkGroth16Vkey::validate_fq2_points(&self.CommitmentKey.G)?;
		GnarkGroth16Vkey::validate_fq2_points(&self.CommitmentKey.GRootSigmaNeg)?;

		if !(self.G1.K.len() as u8 == num_public_inputs+1 || self.G1.K.len() as u8 == num_public_inputs+2) {
			return Err(anyhow!("not valid"));
		}

		if self.G1.K.len() as u8 == num_public_inputs +1 && self.PublicAndCommitmentCommitted.len() != 0{
			return Err(anyhow!("not valid"));
		}
		// TODO: ?
		// if self.G1.K.len() as u8 == num_public_inputs + 2 &&
		// 	(self.PublicAndCommitmentCommitted.len() != 1 || self.PublicAndCommitmentCommitted[0].len() !=0){
		// 	return Err(anyhow!("not valid"));
		// }
		info!("vkey validated");
		Ok(())
	}

	fn keccak_hash(&self) -> AnyhowResult<[u8;32]> {
		let mut keccak_ip = Vec::<u8>::new();
		// -- G1 --
		// -- alpha --
		keccak_ip.extend(self.G1.Alpha.X.as_bytes().iter().cloned());
		keccak_ip.extend(self.G1.Alpha.Y.as_bytes().iter().cloned());
		// -- K --
		for i in 0..self.G1.K.len() {
			keccak_ip.extend(self.G1.K[i].X.as_bytes().iter().cloned());
			keccak_ip.extend(self.G1.K[i].Y.as_bytes().iter().cloned());
		}
		// -- G2 --
		// -- beta --
		keccak_ip.extend(self.G2.Beta.X.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Beta.X.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Beta.Y.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Beta.Y.A1.as_bytes().iter().cloned());
		// -- gamma --
		keccak_ip.extend(self.G2.Gamma.X.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Gamma.X.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Gamma.Y.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Gamma.Y.A1.as_bytes().iter().cloned());
		// -- delta --
		keccak_ip.extend(self.G2.Delta.X.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Delta.X.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Delta.Y.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.G2.Delta.Y.A1.as_bytes().iter().cloned());

		// -- CommitmentKey --
		keccak_ip.extend(self.CommitmentKey.G.X.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.G.X.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.G.Y.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.G.Y.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.GRootSigmaNeg.X.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.GRootSigmaNeg.X.A1.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.GRootSigmaNeg.Y.A0.as_bytes().iter().cloned());
		keccak_ip.extend(self.CommitmentKey.GRootSigmaNeg.Y.A1.as_bytes().iter().cloned());

		let vk_hash = keccak(keccak_ip).0;
		Ok(vk_hash)
	}
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Proof {
	Ar: Fq,
	Krs: Fq,
	Bs: Fq2,
	Commitments: Vec<Fq>,
	CommitmentPok: Fq,
}

impl Proof for GnarkGroth16Proof {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: GnarkGroth16Proof = BorshDeserialize::deserialize(bytes)?;
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
		let gnark_proof: GnarkGroth16Proof = serde_json::from_str(&json_data)?;
		Ok(gnark_proof)
	}
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Pis(pub Vec<String>);

impl Pis for GnarkGroth16Pis {
	fn serialize(&self) -> AnyhowResult<Vec<u8>> {
		let mut buffer: Vec<u8> = Vec::new();
		BorshSerialize::serialize(&self,&mut buffer)?;
		Ok(buffer)
	}

	fn deserialize(bytes: &mut &[u8]) -> AnyhowResult<Self> {
		let key: GnarkGroth16Pis = BorshDeserialize::deserialize(bytes)?;
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
		let gnark_pis: GnarkGroth16Pis = serde_json::from_str(&json_data)?;
		Ok(gnark_pis)
	}

	fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
		let mut keccak_ip = Vec::<u8>::new();
		for i in 0..self.0.len() {
			keccak_ip.extend(self.0[i].as_bytes().iter().cloned());
		}
		let hash = keccak(keccak_ip).0;
		Ok(hash)
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