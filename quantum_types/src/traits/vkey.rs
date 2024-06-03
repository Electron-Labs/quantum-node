// use ;
pub type Result<T> = std::io::Result<T>;

use anyhow::Result as AnyhowResult;

pub trait Vkey: Sized {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self>;
    fn dump_vk(&self, path: &str) -> AnyhowResult<()>;
    fn read_vk(full_path: &str) -> AnyhowResult<Self>;
    fn validate(&self, num_public_inputs: u8) -> AnyhowResult<()>;
    fn keccak_hash(&self) -> AnyhowResult<[u8;32]>;
}