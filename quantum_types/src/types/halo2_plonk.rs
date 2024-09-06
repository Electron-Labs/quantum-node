use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use ark_ff::BigInt;
use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use num_bigint::BigUint;
use quantum_utils::error_line;
use quantum_utils::file::read_bytes_from_file;
use quantum_utils::file::write_bytes_to_file;
use quantum_utils::keccak::convert_string_to_be_bytes;
use serde::{Deserialize, Serialize};
use snark_verifier_sdk::snark_verifier::halo2_base::halo2_proofs::halo2curves::bn256::Fr;
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
        let transcript_initial_state_fr = protocol
            .transcript_initial_state
            .ok_or_else(|| anyhow!(error_line!("protocol.transcript_initial_state")))?;
        let mut transcript_initial_state_bytes = transcript_initial_state_fr.to_bytes_le(); // in le
        transcript_initial_state_bytes.reverse(); // in be

        transcript_initial_state_bytes
            .as_slice()
            .try_into()
            .map_err(|_| {
                anyhow!(error_line!(
                    "transcript_initial_state_bytes.as_slice.try_into"
                ))
            })
    }

    fn extended_keccak_hash(&self, n_commitments: Option<u8>) -> AnyhowResult<[u8; 32]> {
        self.keccak_hash()
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let keccak_h = self.keccak_hash()?;
        let mut keccak_ip = Vec::<u8>::new();
        keccak_ip.extend(keccak_h.to_vec());
        for elm in circuit_verifying_id {
            keccak_ip.extend(elm.to_be_bytes());
        }
        let keccak_h = keccak(keccak_ip.clone());
        Ok(keccak_h.0)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PlonkProof {
    // TODO: change it to protocol_bytes
    pub proof_bytes: Vec<u8>,
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

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PlonkPis(pub Vec<u8>);

impl Pis for Halo2PlonkPis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PlonkPis =
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
        Ok(Halo2PlonkPis(pis_bytes))
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let mut keccak_ip = Vec::<u8>::new();

        for pub_str in self.get_data()? {
            keccak_ip.extend(convert_string_to_be_bytes(&pub_str));
        }
        let hash = keccak(keccak_ip);
        Ok(hash.0)
    }

    fn extended_keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        self.keccak_hash()
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        let a: Vec<Vec<Fr>> = serde_json::from_str(&String::from_utf8(self.0.clone()).map_err(|err| anyhow!(error_line!(err)))?).map_err(|e| anyhow!(error_line!(e)))?;
        let pis = a
            .iter()
            .flat_map(|fr| {
                fr.iter()
                    .map(|elm| {
                        let bytes = elm.to_bytes_le();
                        BigUint::from_bytes_le(&bytes).to_string()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Ok(pis)
    }
}
