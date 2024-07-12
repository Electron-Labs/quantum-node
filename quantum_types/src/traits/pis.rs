use anyhow::Result as AnyhowResult;

pub trait Pis: Sized {
    fn get_data(&self) -> AnyhowResult<Vec<String>>;
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self>;
    fn dump_pis(&self,path: &str) -> AnyhowResult<()>;
    fn read_pis(full_path: &str) -> AnyhowResult<Self>;
    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]>;
    fn extended_keccak_hash(&self) -> AnyhowResult<[u8;32]>;
}