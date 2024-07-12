pub fn get_user_vk_path(storage_folder_path: &str, user_data_path: &str, circuit_hash: &str) -> String {
    let vk_path = format!("{}/{}{}", storage_folder_path, circuit_hash, user_data_path);
    let vkey_full_path = format!("{}/vkey.bin", vk_path.as_str() );
    vkey_full_path
}

pub fn get_user_proof_path(storage_folder_path: &str, proof_path: &str, circuit_hash: &str, proof_id: &str) -> String {
    let proof_path = format!("{}/{}{}", storage_folder_path, circuit_hash, proof_path);
    let file_name = format!("proof_{}.bin", proof_id);
    let proof_key_full_path = format!("{}/{}", proof_path.as_str(), &file_name);
    proof_key_full_path
}

pub fn get_user_pis_path(storage_folder_path: &str, public_inputs_path: &str, circuit_hash: &str, proof_id: &str) -> String {
    let pis_path = format!("{}/{}{}", storage_folder_path, circuit_hash, public_inputs_path);
    // TODO: make it .bin
    let file_name = format!("pis_{}.json", proof_id);
    let pis_key_full_path = format!("{}/{}", pis_path.as_str(), &file_name);
    pis_key_full_path
}

pub fn get_reduction_circuit_proving_key_path(storage_folder_path: &str, reduced_circuit_path: &str, circuit_id: &str) -> String {
    let reduced_circuit_path = format!("{}{}/{}", storage_folder_path, reduced_circuit_path, circuit_id);
    let pk_path = format!("{}/{}", &reduced_circuit_path, "pk.bin");
    pk_path
}

pub fn get_reduction_circuit_verifying_key_path(storage_folder_path: &str, reduced_circuit_path: &str, circuit_id: &str) -> String {
    let reduced_circuit_path = format!("{}{}/{}", storage_folder_path, reduced_circuit_path, circuit_id);
    let vk_path = format!("{}/{}", &reduced_circuit_path, "vk.bin");
    vk_path
}

pub fn get_reduction_circuit_proof_path(storage_folder_path: &str, reduced_proof_path: &str, circuit_hash: &str, proof_id: &str) -> String {
    let reduced_proof_dir = format!("{}/{}{}", storage_folder_path, circuit_hash, reduced_proof_path);
    let proof_path = format!("{}/reduced_proof_{}.bin", reduced_proof_dir, proof_id);
    proof_path
}

pub fn get_reduction_circuit_pis_path(storage_folder_path: &str, reduced_pis_path: &str, circuit_hash: &str, proof_id: &str) -> String {
    let reduced_pis_dir = format!("{}/{}{}", storage_folder_path, circuit_hash, reduced_pis_path);
    let pis_path = format!("{}/reduced_pis_{}.bin", reduced_pis_dir, proof_id);
    pis_path
}

pub fn get_superproof_proof_path(storage_folder_path: &str, superproof_path: &str, superproof_id: u64) -> String {
    format!("{}{}/{}/proof.bin", storage_folder_path, superproof_path, superproof_id)
}

pub fn get_superproof_pis_path(storage_folder_path: &str, superproof_path: &str, superproof_id: u64) -> String {
    format!("{}{}/{}/pis.bin", storage_folder_path, superproof_path, superproof_id)
}

pub fn get_superproof_leaves_path(storage_folder_path: &str, superproof_path: &str, superproof_id: u64) -> String {
    format!("{}{}/{}/leaves.bin", storage_folder_path, superproof_path, superproof_id)
}

pub fn get_imt_proof_path(storage_folder_path: &str, imt_circuit_data_path: &str, superproof_id: u64) -> String {
    format!("{}{}/{}/proof.bin", storage_folder_path, imt_circuit_data_path, superproof_id)
}

pub fn get_imt_pis_path(storage_folder_path: &str, imt_circuit_data_path: &str, superproof_id: u64) -> String {
    format!("{}{}/{}/pis.bin", storage_folder_path, imt_circuit_data_path, superproof_id)
}

pub fn get_imt_vkey_path(aggregated_circuit_data_path: &str) -> String {
    format!("{}/imt_vkey.bin", aggregated_circuit_data_path)
}