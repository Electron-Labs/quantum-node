#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use aggregation::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use gnark_bn254_verifier::{load_plonk_verifying_key_from_bytes, verify};
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
};
use serde::{Deserialize, Serialize};
use utils::{hash::{QuantumHasher, Keccak256Hasher}, public_inputs_hash};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use ark_bn254::Fr as ArkFr;
use std::str::FromStr;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkPlonkVkey {
    pub vkey_bytes: Vec<u8>
}

impl Vkey for GnarkPlonkVkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkPlonkVkey =
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
        let vkey = GnarkPlonkVkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        match load_plonk_verifying_key_from_bytes(&self.vkey_bytes) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let hash = Keccak256Hasher::hash_out(&self.vkey_bytes);
        Ok(hash)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32;8]) -> AnyhowResult<[u8;32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<Keccak256Hasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkPlonkSolidityProof {
    pub proof_bytes: Vec<u8>,
}

impl Proof for GnarkPlonkSolidityProof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkPlonkSolidityProof =
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
        let gnark_proof = GnarkPlonkSolidityProof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }

    fn validate_proof(&self, vkey_path: &str,mut pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vk = GnarkPlonkVkey::read_vk(vkey_path)?;
        let pis = GnarkPlonkPis::deserialize_pis(&mut pis_bytes)?;

        let is_vreified = verify(&self.proof_bytes, &vk.vkey_bytes, &pis.get_ark_pis_for_gnark_plonk_pis()?, gnark_bn254_verifier::ProvingSystem::Plonk);
        if !is_vreified {
            return Err(anyhow!(error_line!("gnark-plonk proof validation failed")))
        }
        Ok(())
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkPlonkPis(pub Vec<String>);

impl Pis for GnarkPlonkPis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: GnarkPlonkPis =
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
        let gnark_pis = GnarkPlonkPis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(gnark_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let ark_pis = self.get_ark_pis_for_gnark_plonk_pis()?;
        let hash = public_inputs_hash::<Keccak256Hasher>(&ark_pis);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}

impl GnarkPlonkPis {
    pub fn get_ark_pis_for_gnark_plonk_pis(&self) ->  AnyhowResult<Vec<ArkFr>> {
        let mut ark_pis = vec![];
    for p in &self.0 {
        ark_pis.push(ArkFr::from_str(&p).map_err(|_| anyhow!(error_line!("failed to form ark pis from snark groth16 pis")))?)
    }
    Ok(ark_pis)
    }
}


#[cfg(test)]
mod tests {
    use crate::{traits::vkey::Vkey, types::gnark_plonk::GnarkPlonkVkey};
    use std::fs;

    #[test]
    pub fn test_keccak_hash() {
        let json_data =
            fs::read_to_string("../test_data/gnark_plonk_vkey.json").expect("Failed to read file");
        let vkey: GnarkPlonkVkey =
            serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

        let expected: [u8; 32] = [
            100, 168, 120, 95, 191, 5, 84, 168, 188, 55, 39, 153, 81, 199, 37, 3, 76, 11, 3, 45,
            227, 85, 91, 23, 12, 192, 120, 242, 136, 16, 171, 213,
        ];
        let computed = vkey.keccak_hash().unwrap();
        assert_eq!(computed, expected);
    }
}
