use anyhow::Result as AnyhowResult;

use crate::types::config::ConfigData;

pub trait Proof: Sized {
    fn serialize(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize(bytes: Vec<u8>) -> AnyhowResult<Self>;
    fn dump_proof(&self, circuit_hash: &str, config: &ConfigData, proof_id: &str) -> AnyhowResult<String>;
    fn read_proof(full_path: &str) -> AnyhowResult<Self>;
}