use hex::ToHex;
use keccak_hash::{keccak, H256};
use anyhow::{Ok, Result as AnyhowResult};

pub fn get_keccak_hash_of_string(value: &str) -> [u8; 32]{
    let hash = keccak(value);
    hash.0
}

pub fn encode_keccak_hash(keccak_bytes: &[u8; 32]) -> AnyhowResult<String> {
    let keccak_h256 = H256::from_slice(keccak_bytes);   
    Ok(format!("0x{}",keccak_h256.encode_hex::<String>()))
}

pub fn decode_keccak_hex(keccak_hex: &str) -> AnyhowResult<[u8; 32]> {
    let mut decoded  = [0; 32];
    hex::decode_to_slice(keccak_hex, &mut decoded)?;
    Ok(decoded)
}