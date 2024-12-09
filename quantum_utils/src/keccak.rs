use std::str::FromStr;

use anyhow::{anyhow, Ok, Result as AnyhowResult};
use halo2curves_axiom::{bn256::Fr, ff::PrimeField};
use hex::ToHex;
use keccak_hash::{keccak, H256};
use num_bigint::BigUint;

use crate::error_line;

pub fn get_keccak_hash_of_string(value: &str) -> [u8; 32] {
    let hash = keccak(value);
    hash.0
}

pub fn encode_keccak_hash(keccak_bytes: &[u8; 32]) -> AnyhowResult<String> {
    let keccak_h256 = H256::from_slice(keccak_bytes);
    Ok(format!("0x{}", keccak_h256.encode_hex::<String>()))
}

pub fn decode_keccak_hex(keccak_hex: &str) -> AnyhowResult<[u8; 32]> {
    let mut decoded = [0; 32];
    hex::decode_to_slice(&keccak_hex[2..], &mut decoded)
        .map_err(|err| anyhow!(error_line!(err)))?;
    Ok(decoded)
}

pub fn convert_string_to_be_bytes(ip: &str) -> [u8; 32] {
    let mut x = BigUint::from_str(ip).unwrap().to_bytes_le();
    while x.len() < 32 {
        x.push(0u8);
    }
    x.reverse();
    x[0..32].try_into().unwrap()
}

pub fn pub_inputs_str_to_fr(pub_inputs: &Vec<String>) -> Vec<Fr> {
    pub_inputs
        .iter()
        .map(|elm| Fr::from_str_vartime(elm).unwrap())
        .collect()
}
