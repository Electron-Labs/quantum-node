use std::fs::File;
use std::io::{Read, Write};
use anyhow::Result as AnyhowResult;
use quantum_circuits_ffi::circuit_builder::GnarkProof;
use quantum_types::types::config::ConfigData;
use quantum_types::types::gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof};
use quantum_utils::file::{create_dir, write_bytes_to_file};
use quantum_utils::keccak::get_keccak_hash_from_bytes;

// Returns circuit_id, pk_path, vk_path
pub fn dump_reduction_circuit_data(config: &ConfigData, pk_bytes_raw: &Vec<u8>, vk_bytes_raw: &Vec<u8>) -> AnyhowResult<(String, String, String)> {
    // Reduction circuit id --> keccak256(vk_bytes_raw)
    let circuit_id = get_keccak_hash_from_bytes(vk_bytes_raw.as_slice());
    let reduced_circuit_path = format!("{}{}/{}", config.storage_folder_path, config.reduced_circuit_path, circuit_id);
    create_dir(&reduced_circuit_path)?;
    let pk_path = format!("{}/{}", &reduced_circuit_path, "pk.bin");
    let vk_path = format!("{}/{}", &reduced_circuit_path, "vk.bin");
    write_bytes_to_file(&pk_bytes_raw, &pk_path)?;
    write_bytes_to_file(&vk_bytes_raw, &vk_path)?;
    Ok((circuit_id.to_string(), pk_path.to_string(), vk_path.to_string()))
}

// Returns reduced_proof_path, reduced_pis_path
pub fn dump_reduction_proof_data(config: &ConfigData, circuit_hash: &str, proof_id: &str, proof_bytes: Vec<u8>, pis_bytes: Vec<u8>) -> AnyhowResult<(String, String)> {
    let reduced_proof_dir = format!("{}/{}{}",config.storage_folder_path, circuit_hash, config.reduced_proof_path);
    let reduced_pis_dir = format!("{}/{}{}",config.storage_folder_path, circuit_hash, config.reduced_pis_path);
    create_dir(&reduced_proof_dir)?;
    create_dir(&reduced_pis_dir)?;
    let proof_path = format!("{}/reduced_proof_{}.bin", reduced_proof_dir, proof_id);
    let pis_path = format!("{}/reduced_pis_{}.bin", reduced_pis_dir, proof_id);
    write_bytes_to_file(&proof_bytes, &proof_path)?;
    write_bytes_to_file(&pis_bytes, &pis_path)?;
    Ok((proof_path.to_string(), pis_path.to_string()))
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