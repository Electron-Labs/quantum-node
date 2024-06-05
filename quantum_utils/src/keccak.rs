use std::str::FromStr;

use hex::ToHex;
use keccak_hash::{keccak, H256};
use anyhow::{Ok, Result as AnyhowResult};
use num_bigint::BigUint;

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

pub fn convert_string_to_le_bytes(ip: &str) -> [u8; 32] {
    let mut x = BigUint::from_str(ip).unwrap().to_bytes_le();
    while x.len() < 32 {
        x.push(0u8);
    }
    x[0..32].try_into().unwrap()
}