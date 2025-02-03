use agg_core::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow , Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::{error_line, file::{read_bytes_from_file, write_bytes_to_file}};
use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use utils::hash::{Hasher, KeccakHasher};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Risc0Vkey {
    pub vkey_bytes: [u32;8]
}

impl Vkey for Risc0Vkey {
    fn serialize_vkey(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> anyhow::Result<Self> {
        let key: Risc0Vkey =
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
        let vkey = Risc0Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let mut output = [0u8; 32];
        for (i, &num) in self.vkey_bytes.iter().enumerate() {
            output[i * 4..(i + 1) * 4].copy_from_slice(&num.to_le_bytes());
        }
        Ok(output)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<KeccakHasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}


#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
pub struct Risc0Proof {
    pub proof_bytes: Vec<u8>
}

impl Proof for Risc0Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Risc0Proof = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_proof(&self, path: &str) -> AnyhowResult<()> {
        let proof_bytes = self.serialize_proof()?;
        write_bytes_to_file(&proof_bytes, path)?;
        Ok(())
    }

    fn read_proof(full_path: &str) -> AnyhowResult<Self> {
        let proof_bytes = read_bytes_from_file(full_path)?;
        let gnark_proof = Risc0Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }
    
    fn validate_proof(&self, vkey_path: &str,mut _pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vkey = Risc0Vkey::read_vk(vkey_path)?;
        self.get_receipt()?.verify(vkey.vkey_bytes)?;
        Ok(())
    }
    
    fn get_proof_bytes(&self) -> AnyhowResult<Vec<u8>> {
        Ok(self.proof_bytes.clone())
    }
}

impl Risc0Proof {
    pub fn get_receipt(&self) -> AnyhowResult<Receipt> {
        let key: Receipt = bincode::deserialize(&self.proof_bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Risc0Pis(pub Vec<String>);

impl Pis for Risc0Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Risc0Pis =
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
        let gnark_pis = Risc0Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(gnark_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let pis_bytes = hex::decode(self.0[0].clone())?;
        let hash = KeccakHasher::hash_out(&pis_bytes);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}
