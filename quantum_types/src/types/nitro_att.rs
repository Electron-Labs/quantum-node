use agg_core::inputs::compute_combined_vkey_hash;
use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use oyster::attestation::AttestationExpectations;
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
};
use serde::{Deserialize, Serialize};
use utils::hash::{Hasher, KeccakHasher};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct NitroAttVkey {
    pub pcr0_bytes: Vec<u8>,
}

impl Vkey for NitroAttVkey {
    fn serialize_vkey(&self) -> anyhow::Result<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> anyhow::Result<Self> {
        let key: NitroAttVkey =
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
        let vkey = NitroAttVkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        if self.pcr0_bytes.len() != 48 {
            return Err(anyhow!("Invalid PCR0 bytes length"));
        }
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        Ok(self.pcr0_bytes[..32]
            .try_into()
            .map_err(|e| anyhow!("invalid pcr0_bytes, {}", e))?)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash =
            compute_combined_vkey_hash::<KeccakHasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
pub struct NitroAttProof {
    pub att_doc_bytes: Vec<u8>,
}

impl Proof for NitroAttProof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: NitroAttProof =
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
        let proof = NitroAttProof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(proof)
    }

    fn validate_proof(&self, vkey_path: &str, mut _pis_bytes: &[u8]) -> AnyhowResult<()> {
        let _ =
            oyster::attestation::verify(&self.att_doc_bytes, AttestationExpectations::default())?;
        Ok(())
    }

    fn get_proof_bytes(&self) -> AnyhowResult<Vec<u8>> {
        Ok(self.att_doc_bytes.clone())
    }
}

impl NitroAttProof {
    pub fn get_pis(&self) -> AnyhowResult<Vec<u8>> {
        let att_decoded =
            oyster::attestation::verify(&self.att_doc_bytes, AttestationExpectations::default())?;

        let mut pis_bytes = vec![];

        pis_bytes.extend_from_slice(&(att_decoded.timestamp as u64).to_be_bytes());
        pis_bytes.extend_from_slice(
            &att_decoded
                .pcrs
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
        );
        pis_bytes.extend_from_slice(&att_decoded.root_public_key);
        pis_bytes.push(att_decoded.public_key.len() as u8);
        pis_bytes.extend_from_slice(&att_decoded.public_key);
        pis_bytes.extend_from_slice(&(att_decoded.user_data.len() as u16).to_be_bytes());
        pis_bytes.extend_from_slice(&att_decoded.user_data);

        Ok(pis_bytes)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct NitroAttPis(pub Vec<String>);

impl Pis for NitroAttPis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: NitroAttPis =
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
        let gnark_pis = NitroAttPis::deserialize_pis(&mut pis_bytes.as_slice())?;
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
