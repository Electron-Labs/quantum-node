use anyhow::Result as AnyhowResult;
use quantum_types::traits::pis::Pis;
use quantum_types::traits::proof::Proof;
use quantum_types::{traits::vkey::Vkey, types::gnark_groth16::GnarkGroth16Proof};
use quantum_types::types::config::ConfigData;
use quantum_types::types::gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Vkey};
use quantum_utils::file::{create_dir, write_bytes_to_file};

// Returns circuit_id, pk_path, vk_path
pub fn dump_reduction_circuit_data(config: &ConfigData, proving_key_bytes: &Vec<u8>, vkey: &GnarkGroth16Vkey) -> AnyhowResult<(String, String, String)> {
    // Reduction circuit id --> keccak256(vk_bytes_raw)
    let vkey_bytes = vkey.serialize()?;
    let circuit_id = String::from_utf8(vkey.keccak_hash()?.to_vec())?;//get_keccak_hash_from_bytes(vk_bytes_raw.as_slice());
    let reduced_circuit_path = format!("{}{}/{}", config.storage_folder_path, config.reduced_circuit_path, circuit_id);
    create_dir(&reduced_circuit_path)?;
    let pk_path = format!("{}/{}", &reduced_circuit_path, "pk.bin");
    let vk_path = format!("{}/{}", &reduced_circuit_path, "vk.bin");
    write_bytes_to_file(&proving_key_bytes, &pk_path)?;
    write_bytes_to_file(&vkey_bytes, &vk_path)?;
    Ok((circuit_id, pk_path.to_string(), vk_path.to_string()))
}

// Returns reduced_proof_path, reduced_pis_path
pub fn dump_reduction_proof_data(config: &ConfigData, circuit_hash: &str, proof_id: &str, proof:GnarkGroth16Proof, pis: GnarkGroth16Pis) -> AnyhowResult<(String, String)> {
    let reduced_proof_dir = format!("{}/{}{}",config.storage_folder_path, circuit_hash, config.reduced_proof_path);
    let reduced_pis_dir = format!("{}/{}{}",config.storage_folder_path, circuit_hash, config.reduced_pis_path);
    create_dir(&reduced_proof_dir)?;
    create_dir(&reduced_pis_dir)?;
    let proof_path = format!("{}/reduced_proof_{}.bin", reduced_proof_dir, proof_id);
    let pis_path = format!("{}/reduced_pis_{}.bin", reduced_pis_dir, proof_id);
    let proof_bytes = proof.serialize()?;
    let pis_bytes = pis.serialize()?;
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