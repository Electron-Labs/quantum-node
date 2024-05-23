use keccak_hash::keccak;

pub fn get_keccak_hash_from_bytes(bytes: &[u8]) -> String{
    let hash = keccak(bytes);
    let hash_string = format!("{:?}", hash);
    hash_string
}