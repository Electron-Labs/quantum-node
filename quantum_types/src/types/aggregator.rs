use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};

use super::gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey};
use anyhow::{Ok, Result as AnyhowResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct InnerCircuitData {
    proof: GnarkGroth16Proof,
    pis: GnarkGroth16Pis,
    vkey: GnarkGroth16Vkey
}

impl InnerCircuitData {
    pub fn construct_from_paths(proof_path: &str, pis_path: &str, vkey_path: &str) -> AnyhowResult<InnerCircuitData>{
        let proof = GnarkGroth16Proof::read_proof(proof_path)?;
        let pis = GnarkGroth16Pis::read_pis(pis_path)?;
        let vkey = GnarkGroth16Vkey::read_vk(vkey_path)?;
        Ok(InnerCircuitData{
            proof,
            pis,
            vkey,
        })
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AggregatorCircuitData {
    proving_key_bytes: Vec<u8>,
    verifying_key_bytes: Vec<u8>
}

impl AggregatorCircuitData {
    pub fn dump_data(pk_path: &str, vk_path: &str) -> AnyhowResult<()> {
        Ok(())
    }

    pub fn read_data(pk_path: &str, vk_path: &str) -> AnyhowResult<AggregatorCircuitData> {
        let data = AggregatorCircuitData { proving_key_bytes: todo!(), verifying_key_bytes: todo!() };
        Ok(data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Leaf {
    pub value: Vec<u8>,
    pub next_value: Vec<u8>,
    pub next_idx: Vec<u8>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct IMTLeaves (pub Vec<Leaf>);

impl IMTLeaves {
    pub fn serialize(&self) -> AnyhowResult<Vec<u8>> {
       Ok(serde_json::to_vec(&self)?)
    }

    pub fn deserialize(serialized_vec: &Vec<u8>) -> AnyhowResult<IMTLeaves> {
        let imt_leaves: IMTLeaves = serde_json::from_slice(&serialized_vec)?;
        Ok(imt_leaves)
    }
}

pub struct AggregatorData {
    inner_circuit_data: Vec<InnerCircuitData>,
    aggregator_circuit_data: AggregatorCircuitData,
    current_leaves: Vec<Leaf>
}

