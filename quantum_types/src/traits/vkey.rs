// use ;
pub type Result<T> = std::io::Result<T>;

use anyhow::Result as AnyhowResult;
// use sqlx::Any;

pub trait Vkey: Sized {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self>;
    fn dump_vk(&self, path: &str) -> AnyhowResult<()>;
    fn read_vk(full_path: &str) -> AnyhowResult<Self>;
    fn validate(&self) -> AnyhowResult<()>;
    fn keccak_hash(&self) -> AnyhowResult<[u8;32]>;
    fn compute_circuit_hash(&self, circuit_verifying_id: [u32;8]) -> AnyhowResult<[u8;32]>;
}