use anyhow::Result as AnyhowResult;

pub trait Proof: Sized {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>>;
    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self>;
    fn dump_proof(&self, path: &str) -> AnyhowResult<()>;
    fn read_proof(full_path: &str) -> AnyhowResult<Self>;
    fn validate_proof(&self, vkey_path: &str, pis_bytes: &[u8]) -> AnyhowResult<()>;
    fn get_proof_bytes(&self) -> AnyhowResult<Vec<u8>>;
}