use agg_core::inputs::get_init_tree_data;
use anyhow::anyhow;
use anyhow::Result as AnyhowResult;
use imt_core::types::Leaf;
use quantum_db::repository::superproof_repository::get_last_verified_superproof;
use quantum_types::traits::pis::Pis;
use quantum_types::traits::proof::Proof;
use quantum_types::types::config::ConfigData;
use quantum_types::types::gnark_groth16::GnarkGroth16Pis;
use quantum_types::types::gnark_groth16::GnarkGroth16Proof;
use quantum_utils::error_line;
use quantum_utils::file::dump_object;
use quantum_utils::keccak::encode_keccak_hash;
use quantum_utils::paths::{
    get_imt_pis_path, get_imt_proof_path, get_reduced_proof_receipt_path
};
use risc0_zkvm::Receipt;

use tracing::info;
use utils::hash::Hasher;
use crate::connection::get_pool;

// Returns circuit_id, pk_path, vk_path
// pub fn dump_reduction_circuit_data(
//     config: &ConfigData,
//     proving_key_bytes: &Vec<u8>,
//     vkey: &GnarkGroth16Vkey,
// ) -> AnyhowResult<(String, String, String)> {
//     // Calculate circuit id
//     let circuit_id = encode_keccak_hash(&vkey.keccak_hash()?)?;

//     // Dump proving key bytes
//     let pkey_path = get_reduction_circuit_proving_key_path(
//         &config.storage_folder_path,
//         &config.reduced_circuit_path,
//         &circuit_id,
//     );
//     write_bytes_to_file(&proving_key_bytes, &pkey_path)?;

//     // Dump verification key bytes
//     let vkey_path = get_reduction_circuit_verifying_key_path(
//         &config.storage_folder_path,
//         &config.reduced_circuit_path,
//         &circuit_id,
//     );

//     vkey.dump_vk(&vkey_path)?;

//     Ok((circuit_id, pkey_path, vkey_path))
// }

// Returns reduced_proof_path, reduced_pis_path
pub fn dump_reduction_proof_data(
    config: &ConfigData,
    circuit_hash: &str,
    proof_hash: &str,
    receipt: Receipt
) -> AnyhowResult<String> {
    let receipt_path = get_reduced_proof_receipt_path(
        &config.storage_folder_path,
        &config.reduced_proof_receipt_path,
        circuit_hash,
        proof_hash,
    );
    
    dump_object(receipt, &receipt_path).map_err(|err| anyhow!(error_line!(err)))?;
    Ok(receipt_path)
}

// Returns imt_proof_path, imt_pis_path
pub fn dump_imt_proof_data(
    config: &ConfigData,
    superproof_id: u64,
    proof: GnarkGroth16Proof,
    pis: GnarkGroth16Pis,
) -> AnyhowResult<(String, String)> {
    let proof_path = get_imt_proof_path(
        &config.storage_folder_path,
        &config.imt_circuit_data_path,
        superproof_id,
    );
    let pis_path = get_imt_pis_path(
        &config.storage_folder_path,
        &config.imt_circuit_data_path,
        superproof_id,
    );
    proof.dump_proof(&proof_path)?;
    pis.dump_pis(&pis_path)?;
    Ok((proof_path, pis_path))
}

// returns empty tree root if leaves not found
pub async fn get_last_superproof_leaves<H:Hasher>(
    config: &ConfigData,
) -> AnyhowResult<Vec<Leaf<H>>> {
    let some_superproof = get_last_verified_superproof(get_pool().await).await?;
    let last_leaves: Vec<Leaf<H>>;
    match some_superproof {
        Some(superproof) => match superproof.superproof_leaves_path {
            Some(superproof_leaves_path) => {
                last_leaves = bincode::deserialize(&std::fs::read(&superproof_leaves_path)?)?;
            }
            _ => {
                info!(
                    "No superproof_leaves_path for superproof_id={} => using last empty tree root",
                    superproof.id.unwrap() // can't be null
                );
                (last_leaves, _) = get_init_tree_data::<H>(config.imt_depth as u8)?;
            }
        },
        // TODO: handle case when we shift to risc0, we dont want to read last superproof leaf(in prod);
        _ => {
            info!("No superproof => using last empty tree root");
            (last_leaves, _) = get_init_tree_data::<H>(config.imt_depth as u8)?;
        }
    }
    Ok(last_leaves)
}

#[cfg(test)]
mod tests {
    use quantum_utils::file::{read_bytes_from_file, write_bytes_to_file};

    #[test]
    pub fn test_read_write() {
        let bytes_vec: Vec<u8> = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        write_bytes_to_file(&bytes_vec, "./test.bytes").expect("Failed to write bytes to file");
        let read_bytes_vec = read_bytes_from_file("./test.bytes").unwrap();
        assert_eq!(read_bytes_vec, bytes_vec);
    }
}
