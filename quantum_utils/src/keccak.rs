use keccak_hash::keccak;

pub fn get_keccak_hash_from_bytes(bytes: &[u8]) -> String{
    let hash = keccak(bytes);
    let hash_string = format!("{:?}", hash);
    hash_string
}

pub fn get_keccak_hash_of_string(value: &str) -> [u8; 32]{
    let hash = keccak(value);
    hash.0
}

