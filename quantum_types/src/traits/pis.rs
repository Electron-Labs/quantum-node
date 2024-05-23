use anyhow::Result as AnyhowResult;

use crate::types::config::ConfigData;

pub trait Pis: Sized {
    fn serialize(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize(bytes: Vec<u8>) -> AnyhowResult<Self>;
    fn dump_pis(&self, circuit_hash: &str, config: &ConfigData, proof_id: &str) -> AnyhowResult<String>;
    fn read_pis(full_path: &str) -> AnyhowResult<Self>;
}