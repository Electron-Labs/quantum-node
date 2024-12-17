#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::str::FromStr;
use aggregation::inputs::compute_combined_vkey_hash;
use ark_bn254::{Bn254, Fq as ArkFq, Fq2 as ArkFq2, Fr as ArkFr, G1Affine, G2Affine};
use ark_groth16::{verifier, Groth16, Proof as ArkProof, VerifyingKey};
use borsh::{BorshDeserialize, BorshSerialize};
use groth16_verifier::groth16_vkey_hash;
use num_bigint::BigUint;
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
};
use serde::{Deserialize, Serialize};
use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;
use utils::{hash::Keccak256Hasher, public_inputs_hash};

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
    pub IC: Vec<Vec<String>>,
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

    pub fn validate_fq2_point(fq2: &Vec<Vec<String>>) -> AnyhowResult<()> {
        if fq2.len() != 3 || fq2[2].len() != 2 || fq2[2][0] != "1" || fq2[2][1] != "0" {
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
    pub fn get_ark_vk_for_snarkjs_groth16(&self) -> AnyhowResult<VerifyingKey<Bn254>> {
        let alpha_g1 = G1Affine::new(
            ArkFq::from_str(
                &self.vk_alpha_1[0]
            ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ArkFq::from_str(
                &self.vk_alpha_1[1],
            ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
        );
        let beta_g2 = G2Affine::new(
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_beta_2[0][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_beta_2[0][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ),
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_beta_2[1][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_beta_2[1][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ),
        );
        let gamma_g2 = G2Affine::new(
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_gamma_2[0][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_gamma_2[0][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ),
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_gamma_2[1][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_gamma_2[1][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,

            ),
        );
        let delta_g2 = G2Affine::new(
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_delta_2[0][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_delta_2[0][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ),
            ArkFq2::new(
                ArkFq::from_str(
                    &self.vk_delta_2[1][0],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &self.vk_delta_2[1][1],
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            ),
        );

        let mut gamma_abc_g1 = vec![];
        for ic in &self.IC {
            let g1 = G1Affine::new(
                ArkFq::from_str(
                    &ic[0]
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
                ArkFq::from_str(
                    &ic[1]
                ).map_err(|_| anyhow!(error_line!("failed to form ark vk from snark groth16 vk")))?,
            );
            gamma_abc_g1.push(g1);
        }

        let ark_vk = VerifyingKey::<Bn254>{
            alpha_g1,
            beta_g2,
            gamma_g2,
            delta_g2,
            gamma_abc_g1
        };
        Ok(ark_vk)
    }
}

impl Vkey for SnarkJSGroth16Vkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Vkey =
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
        let snarkjs_vkey = SnarkJSGroth16Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(snarkjs_vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
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

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        // let gnark_converted_vkey = self.convert_to_gnark_vkey();

        // Ok(gnark_converted_vkey.keccak_hash()?)
        let ark_vk = self.get_ark_vk_for_snarkjs_groth16()?;
        println!("ark_vk done");
        let pvk_hash = groth16_vkey_hash::<Keccak256Hasher>(&ark_vk);
        Ok(pvk_hash)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        // let gnark_converted_vkey = self.convert_to_gnark_vkey();
        // gnark_converted_vkey.compute_circuit_hash(circuit_verifying_id)
        let pvk_hash = self.keccak_hash()?;

        let circuit_hash = compute_combined_vkey_hash::<Keccak256Hasher>(&pvk_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Proof {
    pub pi_a: Vec<String>,
    pub pi_b: Vec<Vec<String>>,
    pub pi_c: Vec<String>,
    pub protocol: String,
    pub curve: String,
}

impl Proof for SnarkJSGroth16Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Proof =
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
        let snarkjs_proof = SnarkJSGroth16Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(snarkjs_proof)
    }

    fn validate_proof(&self, vkey_path: &str, mut pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vkey = SnarkJSGroth16Vkey::read_vk(vkey_path)?;
        let pis = SnarkJSGroth16Pis::deserialize_pis(&mut pis_bytes)?;
        let ark_vk = vkey.get_ark_vk_for_snarkjs_groth16()?;
        let pvk = verifier::prepare_verifying_key(&ark_vk);

        let ark_proof = self.get_ark_proof_for_snarkjs_groth16_proof()?;
        let ark_pis = pis.get_ark_pis_for_snarkjs_groth16_pis()?;

        let res = Groth16::<Bn254>::verify_proof(&pvk, &ark_proof, &ark_pis).map_err(|e| {anyhow!(error_line!(format!("error while validating proof: {}", e)))})?;
        if !res {
            return Err(anyhow!(error_line!("snarkJS-groth16 proof validation failed")))
        }
        Ok(())
    }
}

impl SnarkJSGroth16Proof {
    pub fn get_ark_proof_for_snarkjs_groth16_proof(&self) -> AnyhowResult<ArkProof<Bn254>> {
        let a = G1Affine::new(
            ArkFq::from_str(
                &self.pi_a[0]
            ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
            ArkFq::from_str(
                &self.pi_a[1]
            ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
        );
        let b = G2Affine::new(
            ArkFq2::new(
                ArkFq::from_str(
                    &self.pi_b[0][0]
                ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
                ArkFq::from_str(
                    &self.pi_b[0][1]
                ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
            ),
            ArkFq2::new(
                ArkFq::from_str(
                    &self.pi_b[1][0]
                ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
                ArkFq::from_str(
                    &self.pi_b[1][1]
                ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
            ),
        );
        let c = G1Affine::new(
            ArkFq::from_str(
                &self.pi_c[0]
            ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
            ArkFq::from_str(
                &self.pi_c[1]
            ).map_err(|_| anyhow!(error_line!("failed to form ark proof from snark groth16 proof")))?,
        );
        let ark_proof = ArkProof::<Bn254> { a, b, c };
        Ok(ark_proof)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct SnarkJSGroth16Pis(pub Vec<String>);

impl Pis for SnarkJSGroth16Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: SnarkJSGroth16Pis =
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
        let snarkjs_pis: SnarkJSGroth16Pis =
            SnarkJSGroth16Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(snarkjs_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let ark_pis = self.get_ark_pis_for_snarkjs_groth16_pis()?;
        let hash = public_inputs_hash::<Keccak256Hasher>(&ark_pis);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}

impl SnarkJSGroth16Pis {
    pub fn get_ark_pis_for_snarkjs_groth16_pis(&self) ->  AnyhowResult<Vec<ArkFr>> {
        let mut ark_pis = vec![];
    for p in &self.0 {
        ark_pis.push(ArkFr::from_str(&p).map_err(|_| anyhow!(error_line!("failed to form ark pis from snark groth16 pis")))?)
    }
    Ok(ark_pis)
    }
}

#[cfg(test)]
mod tests {
    use borsh::{BorshDeserialize, BorshSerialize};
    use std::fs;

    use super::SnarkJSGroth16Vkey;

    #[test]
    pub fn serde_test() {
        let json_data = fs::read_to_string("./dumps/circom1_vk.json").expect("Failed to read file");
        let snarkjs_vkey: SnarkJSGroth16Vkey =
            serde_json::from_str(&json_data).expect("Failed to deserialize JSON data");

        let mut buffer: Vec<u8> = Vec::new();
        snarkjs_vkey.serialize(&mut buffer).unwrap();
        println!("serialised vkey {:?}", buffer);

        let re_snarkjs_vkey = SnarkJSGroth16Vkey::deserialize(&mut &buffer[..]).unwrap();

        assert_eq!(snarkjs_vkey, re_snarkjs_vkey);
    }
}
