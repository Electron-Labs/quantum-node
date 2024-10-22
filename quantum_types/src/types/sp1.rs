use agg_core::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow , Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::{error_line, file::{read_bytes_from_file, write_bytes_to_file}};
use serde::{Deserialize, Serialize};
use sp1_prover::types::SP1VerifyingKey;
use sp1_sdk::{HashableKey, SP1ProofWithPublicValues};
use utils::hash::{Hasher, KeccakHasher};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

#[derive(Clone,BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Sp1Vkey {
    pub vkey_bytes: Vec<u8>
}

impl Vkey for Sp1Vkey {
    fn serialize_vkey(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> anyhow::Result<Self> {
        let key: Sp1Vkey = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_vk(&self, path: &str) -> AnyhowResult<()> {
        let vkey_bytes = self.serialize_vkey()?;
        write_bytes_to_file(&vkey_bytes, path)?;
        Ok(())
    }

    fn read_vk(full_path: &str) -> AnyhowResult<Self> {
        let vkey_bytes = read_bytes_from_file(full_path)?;
        let vkey = Sp1Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        Ok(self.get_verifying_key()?.hash_bytes())
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<KeccakHasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

impl Sp1Vkey {
    pub fn get_verifying_key(&self) -> AnyhowResult<SP1VerifyingKey> {
        let vkey = bincode::deserialize(&self.vkey_bytes)?;
        Ok(vkey)
    }
}


#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
pub struct Sp1Proof {
    pub proof_bytes: Vec<u8>
}

impl Proof for Sp1Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Sp1Proof = BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_proof(&self, path: &str) -> AnyhowResult<()> {
        let proof_bytes = self.serialize_proof()?;
        write_bytes_to_file(&proof_bytes, path)?;
        Ok(())
    }

    fn read_proof(full_path: &str) -> AnyhowResult<Self> {
        let proof_bytes = read_bytes_from_file(full_path)?;
        let gnark_proof = Sp1Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }
    
    fn validate_proof(&self, vkey_path: &str,mut _pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vkey = Sp1Vkey::read_vk(vkey_path)?;
        let client = sp1_sdk::ProverClient::local();
        client.verify(&self.get_proof_with_public_inputs()?, &vkey.get_verifying_key()?)?;
        Ok(())
    }
}

impl Sp1Proof {
    pub fn get_proof_with_public_inputs(&self) -> AnyhowResult<SP1ProofWithPublicValues> {
        let proof = bincode::deserialize(&self.proof_bytes)?;
        Ok(proof)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Sp1Pis(pub Vec<String>);

impl Pis for Sp1Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Sp1Pis =
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
        let gnark_pis = Sp1Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(gnark_pis)
    }

    // TODO: ask
    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let pis_bytes = hex::decode(self.0[0].clone())?;
        let hash = KeccakHasher::hash_out(&pis_bytes);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}
