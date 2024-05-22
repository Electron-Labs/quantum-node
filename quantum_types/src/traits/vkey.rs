// use ;
pub type Result<T> = std::io::Result<T>;

use anyhow::Result as AnyhowResult;
pub trait Vkey: Sized {
    fn serialize(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize(bytes: Vec<u8>) -> AnyhowResult<Self>;
    fn dump_vk(&self, circuit_hash: &str, storage_path: &str, user_data_path: &str) -> AnyhowResult<String>;
    fn read_vk(full_path: &str) -> AnyhowResult<Self>;
}