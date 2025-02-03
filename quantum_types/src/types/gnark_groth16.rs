#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;

use agg_core::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use gnark_bn254_verifier::{load_groth16_verifying_key_from_bytes, verify};
use quantum_circuits_interface::ffi::circuit_builder::{self, G1, G1A, G2};
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file , write_bytes_to_file},
};
use serde::{Deserialize, Serialize};
use utils::{hash::{Hasher, KeccakHasher}, public_inputs_hash};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use ark_bn254::Fr as ArkFr;
pub const MAX_PUB_INPUTS: usize = 20;
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
    pub Y: String,
}

impl Fq {
    pub fn zero() -> Self {
        Self {
            X: "0".to_string(),
            Y: "0".to_string(),
        }
    }

    pub fn from_risc_circuit_G1(g1: &G1) -> Self {
        Fq {
            X: g1.X.clone(),
            Y: g1.Y.clone(),
        }
    }
}

#[derive(Clone, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq_2 {
    pub A0: String,
    pub A1: String,
}

impl Fq_2 {
    pub fn from_risc_circuit_G1A(g1A: &G1A) -> Self {
        Fq_2 {
            A0: g1A.A0.clone(),
            A1: g1A.A1.clone(),
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Fq2 {
    pub X: Fq_2,
    pub Y: Fq_2,
}

impl Fq2 {
    pub fn from_risc_circuit_g2(g2: &G2) -> Self {
        Fq2 {
            X: Fq_2::from_risc_circuit_G1A(&g2.X),
            Y: Fq_2::from_risc_circuit_G1A(&g2.Y),
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G1Struct {
    pub Alpha: Fq,
    pub Beta: Fq,
    pub Delta: Fq,
    pub K: Vec<Fq>,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct G2Struct {
    pub Beta: Fq2,
    pub Delta: Fq2,
    pub Gamma: Fq2,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct PedersenCommitmentKey {
    pub G: Fq2,
    pub GRootSigmaNeg: Fq2,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Vkey {
    pub vkey_bytes: Vec<u8>
}

// impl GnarkGroth16Vkey {
//     pub fn validate_fq_point(fq: &Fq) -> AnyhowResult<()> {
//         let x = ark_bn254::Fq::from(BigUint::from_str(&fq.X)?);
//         let y = ark_bn254::Fq::from(BigUint::from_str(&fq.Y)?);
//         let p = ark_bn254::G1Affine::new_unchecked(x, y);
//         let is_valid = GnarkGroth16Vkey::check_if_g1_point_is_valid(&p);
//         if !is_valid {
//             info!("fq point not valid");
//             return Err(anyhow!(error_line!("fq point is not valid")));
//         }
//         Ok(())
//     }

//     pub fn validate_fq2_points(fq2: &Fq2) -> AnyhowResult<()> {
//         let x1 = ark_bn254::Fq::from(BigUint::from_str(&fq2.X.A0)?);
//         let x2 = ark_bn254::Fq::from(BigUint::from_str(&fq2.X.A1)?);
//         let x = ark_bn254::Fq2::new(x1, x2);

//         let y1 = ark_bn254::Fq::from(BigUint::from_str(&fq2.Y.A0)?);
//         let y2 = ark_bn254::Fq::from(BigUint::from_str(&fq2.Y.A1)?);
//         let y = ark_bn254::Fq2::new(y1, y2);

//         let p = ark_bn254::G2Affine::new(x, y);
//         let is_valid = GnarkGroth16Vkey::check_if_g2_point_is_valid(&p);
//         if !is_valid {
//             info!("fq2 point not valid");
//             return Err(anyhow!(error_line!("fq point is not valid")));
//         }
//         Ok(())
//     }

//     pub fn check_if_g1_point_is_valid(p: &Affine<Config>) -> bool {
//         return p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve();
//     }

//     pub fn check_if_g2_point_is_valid(p: &Affine<ark_bn254::g2::Config>) -> bool {
//         return p.is_on_curve() && p.is_in_correct_subgroup_assuming_on_curve();
//     }
// }

impl Vkey for GnarkGroth16Vkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkGroth16Vkey =
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
        let gnark_vkey = GnarkGroth16Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(gnark_vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        match load_groth16_verifying_key_from_bytes(&self.vkey_bytes)  {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let hash = KeccakHasher::hash_out(&self.vkey_bytes);
        Ok(hash)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<KeccakHasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SuperproofGnarkGroth16Proof {
    pub Ar: Fq,
    pub Krs: Fq,
    pub Bs: Fq2,
    pub Commitments: Vec<Fq>,
    pub CommitmentPok: Fq,
}

impl Proof for SuperproofGnarkGroth16Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SuperproofGnarkGroth16Proof =
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
        let gnark_proof = SuperproofGnarkGroth16Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }

    fn validate_proof(&self, _vkey_path: &str, _pis_bytes: &[u8]) -> AnyhowResult<()> {
        Ok(())
    }
    
    fn get_proof_bytes(&self) -> AnyhowResult<Vec<u8>> {
        self.serialize_proof()
    }
}

impl SuperproofGnarkGroth16Proof {
    pub fn from_gnark_proof_result(gnark_proof: circuit_builder::GnarkGroth16Proof) -> Self {
        let commitments = gnark_proof.Commitments
            .iter()
            .map(|g1| Fq::from_risc_circuit_G1(&g1))
            .collect();

            SuperproofGnarkGroth16Proof {
            Ar: Fq::from_risc_circuit_G1(&gnark_proof.Ar),
            Krs: Fq::from_risc_circuit_G1(&gnark_proof.Krs),
            Bs: Fq2::from_risc_circuit_g2(&gnark_proof.Bs),
            Commitments: commitments,
            CommitmentPok: Fq::from_risc_circuit_G1(&gnark_proof.CommitmentPok),
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Proof {
    pub proof_bytes: Vec<u8>,
}

impl Proof for GnarkGroth16Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkGroth16Proof =
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
        let gnark_proof = GnarkGroth16Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }

    fn validate_proof(&self, vkey_path: &str, mut pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vk = GnarkGroth16Vkey::read_vk(vkey_path)?;
        let pis = GnarkGroth16Pis::deserialize_pis(&mut pis_bytes)?;

        let is_vreified = verify(&self.proof_bytes, &vk.vkey_bytes, &pis.get_ark_pis_for_gnark_groth16_pis()?, gnark_bn254_verifier::ProvingSystem::Groth16);
        if !is_vreified {
            return Err(anyhow!(error_line!("gnark-groth16 proof validation failed")))
        }
        Ok(())
    }
    
    fn get_proof_bytes(&self) -> AnyhowResult<Vec<u8>> {
        Ok(self.proof_bytes.clone())
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkGroth16Pis(pub Vec<String>);

impl Pis for GnarkGroth16Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkGroth16Pis =
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
        let gnark_pis = GnarkGroth16Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(gnark_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let ark_pis = self.get_ark_pis_for_gnark_groth16_pis()?;
        let hash = public_inputs_hash::<KeccakHasher>(&ark_pis);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}

impl GnarkGroth16Pis {
    pub fn get_ark_pis_for_gnark_groth16_pis(&self) ->  AnyhowResult<Vec<ArkFr>> {
        let mut ark_pis = vec![];
    for p in &self.0 {
        ark_pis.push(ArkFr::from_str(&p).map_err(|_| anyhow!(error_line!("failed to form ark pis from snark groth16 pis")))?)
    }
    Ok(ark_pis)
    }
}

#[cfg(test)]
mod tests {
    use super::GnarkGroth16Vkey;
    use borsh::{BorshDeserialize, BorshSerialize};
    use std::fs;

    #[test]
    pub fn serde_test() {
        // Read JSON -> Get Struct -> Borsh Serialise -> Borsh Deserialise -> match
        let json_data = fs::read_to_string("./dumps/gnark_vkey.json").expect("Failed to read file");
        let gnark_vkey: GnarkGroth16Vkey =
            serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

        let mut buffer: Vec<u8> = Vec::new();
        gnark_vkey.serialize(&mut buffer).unwrap();
        println!("serialised vkey {:?}", buffer);

        let re_gnark_vkey = GnarkGroth16Vkey::deserialize(&mut &buffer[..]).unwrap();

        assert_eq!(gnark_vkey, re_gnark_vkey);

        println!("{:?}", re_gnark_vkey);
    }
}
