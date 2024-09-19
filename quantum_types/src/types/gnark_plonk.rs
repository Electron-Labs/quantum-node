#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use num_bigint::BigUint;
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
    keccak::convert_string_to_be_bytes,
};
use serde::{Deserialize, Serialize};

use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

use super::gnark_groth16::{Fq, Fq2};

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct KZGVK {
    pub G2: Vec<Fq2>, // [G₂, [α]G₂ ]
    pub G1: Fq,
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkPlonkVkey {
    // Size circuit
    pub Size: u64,
    pub SizeInv: String,
    pub Generator: String,
    pub NbPublicVariables: u64,

    // Commitment scheme that is used for an instantiation of PLONK
    pub Kzg: KZGVK,

    // cosetShift generator of the coset on the small domain
    pub CosetShift: u64,

    // S commitments to S1, S2, S3
    pub S: Vec<Fq>,

    // Commitments to ql, qr, qm, qo, qcp prepended with as many zeroes (ones for l) as there are public inputs.
    // In particular Qk is not complete.
    pub Ql: Fq,
    pub Qr: Fq,
    pub Qm: Fq,
    pub Qo: Fq,
    pub Qk: Fq,
    pub Qcp: Vec<Fq>,

    pub CommitmentConstraintIndexes: Vec<u64>,
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

    fn validate(&self, num_public_inputs: u8) -> AnyhowResult<()> {
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let mut keccak_ip = Vec::<u8>::new();

        // Kzg
        for i in 0..self.Kzg.G2.len() {
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Kzg.G2[i].X.A0)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Kzg.G2[i].X.A1)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Kzg.G2[i].Y.A0)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Kzg.G2[i].Y.A1)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
        }
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Kzg.G1.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Kzg.G1.Y)
                .to_vec()
                .iter()
                .cloned(),
        );

        // CosetShift
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.CosetShift.to_string())
                .to_vec()
                .iter()
                .cloned(),
        );

        // Size
        keccak_ip.extend(self.Size.to_be_bytes());

        // SizeInv
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.SizeInv)
                .to_vec()
                .iter()
                .cloned(),
        );

        // Generator
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Generator)
                .to_vec()
                .iter()
                .cloned(),
        );

        // S
        for i in 0..self.S.len() {
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.S[i].X)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.S[i].Y)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
        }

        // Ql, Qr, Qm, Qo, Qk
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Ql.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Ql.Y)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qr.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qr.Y)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qm.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qm.Y)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qo.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qo.Y)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qk.X)
                .to_vec()
                .iter()
                .cloned(),
        );
        keccak_ip.extend(
            convert_string_to_be_bytes(&self.Qk.Y)
                .to_vec()
                .iter()
                .cloned(),
        );

        // Qcp
        for i in 0..self.Qcp.len() {
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Qcp[i].X)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
            keccak_ip.extend(
                convert_string_to_be_bytes(&self.Qcp[i].Y)
                    .to_vec()
                    .iter()
                    .cloned(),
            );
        }

        // CommitmentConstraintIndexes
        for i in 0..self.CommitmentConstraintIndexes.len() {
            keccak_ip.extend(self.CommitmentConstraintIndexes[i].to_be_bytes());
        }

        let keccak_h = keccak(keccak_ip.clone());
        Ok(keccak_h.0)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32;8]) -> AnyhowResult<[u8;32]> {
        todo!()
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct GnarkPlonkSolidityProof {
    pub ProofBytes: Vec<u8>,
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
        let mut keccak_ip = Vec::<u8>::new();

        for pub_str in self.0.clone() {
            keccak_ip.extend(convert_string_to_be_bytes(&pub_str));
        }
        let hash = keccak(keccak_ip);
        Ok(hash.0)
    }

    // fn extended_keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
    //     self.keccak_hash()
    // }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
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
