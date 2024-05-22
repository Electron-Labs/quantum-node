use std::fs::File;
use std::io::{Read, Write};
use anyhow::Result as AnyhowResult;
use quantum_types::types::config::ConfigData;
use quantum_utils::file::{create_dir, write_bytes_to_file};

// Returns circuit_id, pk_path, vk_path
pub fn dump_reduction_circuit_data(config: &ConfigData, pk_bytes_raw: &Vec<u8>, vk_bytes_raw: &Vec<u8>) -> AnyhowResult<(String, String, String)> {
    // Reduction circuit id --> keccak256(vk_bytes_raw)
    let circuit_id = "hash_circuit_id";
    let reduced_circuit_path = format!("{}{}/{}", config.storage_folder_path, config.reduced_circuit_path, circuit_id);
    create_dir(&reduced_circuit_path)?;
    let pk_path = format!("{}/{}", &reduced_circuit_path, "pk.bin");
    let vk_path = format!("{}/{}", &reduced_circuit_path, "vk.bin");
    write_bytes_to_file(&pk_bytes_raw, &pk_path)?;
    write_bytes_to_file(&vk_bytes_raw, &vk_path)?;
    Ok((circuit_id.to_string(), pk_path.to_string(), vk_path.to_string()))
}


#[cfg(test)]
mod tests {
    use quantum_utils::file::{write_bytes_to_file, read_bytes_from_file};

    #[test]
    pub fn test_read_write() {
        let bytes_vec: Vec<u8> = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        write_bytes_to_file(&bytes_vec, "./test.bytes").expect("Failed to write bytes to file");
        let read_bytes_vec = read_bytes_from_file("./test.bytes").unwrap();
        assert_eq!(read_bytes_vec, bytes_vec);
    }
}