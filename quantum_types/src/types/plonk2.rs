use std::str::FromStr;

use agg_core::inputs::compute_combined_vkey_hash;
use borsh::{BorshDeserialize, BorshSerialize};
use num_bigint::BigUint;
use plonky2::{field::{goldilocks_field::GoldilocksField, types::{Field, PrimeField}}, plonk::{circuit_data::{CommonCircuitData, VerifierCircuitData, VerifierOnlyCircuitData}, config::PoseidonGoldilocksConfig, proof::ProofWithPublicInputs}, util::serialization::DefaultGateSerializer};
use plonky2_core::utils::{plonky2_public_inputs_hash, plonky2_vkey_hash};
use quantum_utils::{error_line, file::{read_bytes_from_file, write_bytes_to_file}};
use serde::{Deserialize, Serialize};
use utils::{hash::KeccakHasher};
use anyhow::{anyhow, Result as AnyhowResult};
use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

type C = PoseidonGoldilocksConfig;
type F = GoldilocksField;
const D: usize = 2;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Plonky2Vkey {
    pub common_bytes: Vec<u8>,
    pub verifier_only_bytes: Vec<u8>,
}

impl Vkey for Plonky2Vkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Plonky2Vkey =
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
        let vkey = Plonky2Vkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        self.get_common_circuit_data()?;
        self.get_verifier_only()?;
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let verifier_only = self.get_verifier_only()?;
        let hash = plonky2_vkey_hash(&verifier_only);
        Ok(hash)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<KeccakHasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

impl Plonky2Vkey {
    pub fn get_verifier_only(&self) -> AnyhowResult<VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>> {
        let verifier_only = VerifierOnlyCircuitData::<C, D>::from_bytes(self.verifier_only_bytes.clone()).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(verifier_only)
    }

    pub fn get_common_circuit_data(&self) -> AnyhowResult<CommonCircuitData<GoldilocksField, 2>> {
        let common = CommonCircuitData::<F, D>::from_bytes(self.common_bytes.clone(), &DefaultGateSerializer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(common)
    }

    pub fn get_verifier(&self) ->AnyhowResult<VerifierCircuitData<GoldilocksField, PoseidonGoldilocksConfig, 2>> {
        let verifier_only = self.get_verifier_only()?;
        let common_circuit_data = self.get_common_circuit_data()?;
        
        let verifier = VerifierCircuitData {
            common: common_circuit_data,
            verifier_only,
        };

        Ok(verifier)
        
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Plonky2Proof {
    // TODO: change it to protocol_bytes
    pub proof_bytes: Vec<u8>,
}

impl Proof for Plonky2Proof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Plonky2Proof =
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
        let gnark_proof = Plonky2Proof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }
    
    fn validate_proof(&self, vkey_path: &str,mut _pis_bytes: &[u8]) -> AnyhowResult<()> {
        let vkey = Plonky2Vkey::read_vk(vkey_path)?;
        let common_circuit_data = vkey.get_common_circuit_data()?;
        let proof_with_pis = self.get_proof_with_pis(&common_circuit_data)?;
        
        // let pis = proof_with_pis.public_inputs;
        let verifier = vkey.get_verifier()?;
        verifier.verify(proof_with_pis)?;
        Ok(())
    }
}

impl Plonky2Proof {
    pub fn get_proof_with_pis(&self, common_circuit_data: &CommonCircuitData<GoldilocksField, 2> ) -> AnyhowResult<ProofWithPublicInputs<GoldilocksField, PoseidonGoldilocksConfig, 2>>{
        let proof_with_pis = ProofWithPublicInputs::from_bytes(self.proof_bytes.clone(), common_circuit_data).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(proof_with_pis)
    }

    pub fn get_pis_strings(&self, vkey_path: &str) -> AnyhowResult<Vec<String>> {
        let vkey = Plonky2Vkey::read_vk(vkey_path)?;
        let common_circuit_data = vkey.get_common_circuit_data()?;
        let proof_with_pis = self.get_proof_with_pis(&common_circuit_data)?;

        let mut pis = vec![];

        for p in proof_with_pis.public_inputs {
            pis.push(p.to_canonical_biguint().to_string());
        }
        Ok(pis)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Plonky2Pis(pub Vec<String>);

impl Pis for Plonky2Pis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Plonky2Pis =
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
        let gnark_pis = Plonky2Pis::deserialize_pis(&mut pis_bytes.as_slice())?;
        Ok(gnark_pis)
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let mut pis = vec![];
        for p in &self.0 {
            pis.push(GoldilocksField::from_noncanonical_biguint(BigUint::from_str(p)?));
        }

        let hash = plonky2_public_inputs_hash::<KeccakHasher>(&pis);
        Ok(hash)
    }

    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        Ok(self.0.clone())
    }
}

// impl Plonky2Pis {
//     pub fn get_from_proof_bytes(proof_bytes: Vec<u8>) {
//         let proof = Plonky2Proof{
//             proof_bytes
//         };

//         let prof_with_pis = 

//     }
// }