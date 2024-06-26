use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use borsh::{BorshDeserialize, BorshSerialize};
use quantum_utils::error_line;
use quantum_utils::file::read_bytes_from_file;
use quantum_utils::file::write_bytes_to_file;
use serde::{Deserialize, Serialize};
use snark_verifier_sdk::snark_verifier::halo2_base::utils::ScalarField;
use snark_verifier_sdk::snark_verifier::{
    halo2_base::halo2_proofs::halo2curves::bn256::G1Affine, verifier::plonk::PlonkProtocol,
};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PlonkVkey {
    pub protocol_bytes: Vec<u8>,
    pub sg2_bytes: Vec<u8>,
    pub proof_bytes: Vec<u8>,
    pub instances_bytes: Vec<u8>,
}

impl Vkey for Halo2PlonkVkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PlonkVkey =
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
        let vkey = Halo2PlonkVkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self, num_public_inputs: u8) -> AnyhowResult<()> {
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let protocol: PlonkProtocol<G1Affine> =
            serde_json::from_str(&String::from_utf8(self.protocol_bytes.clone())?)?;
        let transcript_initial_state = protocol
            .transcript_initial_state
            .ok_or_else(|| anyhow!(error_line!("protocol.transcript_initial_state")))?;
        transcript_initial_state
            .to_bytes_le()
            .try_into()
            .map_err(|_| anyhow!(error_line!("transcript_initial_state.to_bytes_le")))
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PlonkProof {
    pub proof_bytes: Vec<u8>,
    pub instance_bytes: Vec<u8>,
}

impl Proof for Halo2PlonkProof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PlonkProof =
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
        let gnark_proof = Halo2PlonkProof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }
}
