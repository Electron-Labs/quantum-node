use keccak_hash::keccak;

pub fn get_keccal_hash_from_bytes(bytes: Vec<u8>) -> String{
    let mut keccak_ip = bytes.as_slice();
    let hash = keccak(&mut keccak_ip);
    let hash_string = format!("{:?}", hash);
    hash_string
}