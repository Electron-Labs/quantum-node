use keccak_hash::keccak;

pub fn get_keccak_hash_of_string(value: &str) -> [u8; 32]{
    let hash = keccak(value);
    hash.0
}

