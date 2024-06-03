use anyhow::Result as AnyhowResult;
use quantum_types::traits::pis::Pis;
use quantum_types::traits::proof::Proof;
use quantum_types::{traits::vkey::Vkey, types::gnark_groth16::GnarkGroth16Proof};
use quantum_types::types::config::ConfigData;
use quantum_types::types::gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey};
use quantum_utils::file::{create_dir, write_bytes_to_file};
use quantum_utils::paths::{get_reduction_circuit_pis_path, get_reduction_circuit_proof_path, get_reduction_circuit_proving_key_path, get_reduction_circuit_verifying_key_path};

// Returns circuit_id, pk_path, vk_path
pub fn dump_reduction_circuit_data(config: &ConfigData, proving_key_bytes: &Vec<u8>, vkey: &GnarkGroth16Vkey) -> AnyhowResult<(String, String, String)> {
    // Calculate circuit id
    let circuit_id = String::from_utf8(vkey.keccak_hash()?.to_vec())?;

    // Dump proving key bytes
    let pkey_path = get_reduction_circuit_proving_key_path(&config.storage_folder_path, &config.reduced_circuit_path, &circuit_id);
    write_bytes_to_file(&proving_key_bytes, &pkey_path)?;

    // Dump verification key bytes
    let vkey_path = get_reduction_circuit_verifying_key_path(&config.storage_folder_path, &config.reduced_circuit_path, &circuit_id);

    vkey.dump_vk(&vkey_path)?;

    Ok((circuit_id, pkey_path, vkey_path))
}

// Returns reduced_proof_path, reduced_pis_path
pub fn dump_reduction_proof_data(config: &ConfigData, circuit_hash: &str, proof_id: &str, proof:GnarkGroth16Proof, pis: GnarkGroth16Pis) -> AnyhowResult<(String, String)> {
    let proof_path = get_reduction_circuit_proof_path(&config.storage_folder_path, &config.reduced_proof_path, circuit_hash, proof_id);
    let pis_path = get_reduction_circuit_pis_path(&config.storage_folder_path, &config.reduced_pis_path, circuit_hash, proof_id);
    proof.dump_proof(&proof_path)?;
    pis.dump_pis(&pis_path)?;
    Ok((proof_path, pis_path))
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