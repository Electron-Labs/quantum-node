use quantum_circuits_ffi::interactor::{get_init_tree_data, QuantumV2CircuitInteractor};
use quantum_db::repository::{reduction_circuit_repository::get_reduction_circuit_for_user_circuit, superproof_repository::get_last_superproof};
use quantum_types::{traits::{circuit_interactor::{CircuitInteractor, IMT_Tree, KeccakHashOut, QuantumLeaf}, pis::Pis, proof::Proof, vkey::Vkey}, types::{aggregator::{AggregatorCircuitData, IMTLeaves, InnerCircuitData}, config::ConfigData, db::proof::Proof as DBProof, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}}};
use quantum_utils::{file::read_bytes_from_file, keccak::decode_keccak_hex, paths::{get_aggregation_circuit_proving_key_path, get_aggregation_circuit_vkey_path}};
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};

pub const IMT_DEPTH: usize = 10;

pub async fn handle_aggregation(pool: &Pool<MySql>, proofs: Vec<DBProof>,  superproof_id: u64, config: &ConfigData) -> AnyhowResult<()> {
    let mut reduced_proofs = Vec::<GnarkGroth16Proof>::new();
    let mut reduced_pis_vec = Vec::<GnarkGroth16Pis>::new();
    let mut reduced_circuit_vkeys = Vec::<GnarkGroth16Vkey>::new();

    for proof in &proofs {
        let reduced_proof_path = proof.reduction_proof_path.clone().unwrap();
        let reduced_proof = GnarkGroth16Proof::read_proof(&reduced_proof_path)?;
        reduced_proofs.push(reduced_proof);
        let reduced_pis_path = proof.reduction_proof_pis_path.clone().unwrap();
        let reduced_pis = GnarkGroth16Pis::read_pis(&reduced_pis_path)?;
        reduced_pis_vec.push(reduced_pis);
        let reduced_circuit_vkey_path = get_reduction_circuit_for_user_circuit(pool, &proof.user_circuit_hash).await?.vk_path;
        let reduced_vkey = GnarkGroth16Vkey::read_vk(&reduced_circuit_vkey_path)?;
        reduced_circuit_vkeys.push(reduced_vkey);
    }

    let last_updated_superproof = get_last_superproof(pool).await?;
    let last_root: KeccakHashOut;
    let last_leaves: IMT_Tree;
    if last_updated_superproof.is_some() {
        let last_superproof = last_updated_superproof.unwrap();
        last_root = KeccakHashOut(decode_keccak_hex(&last_superproof.superproof_root.unwrap())?);
        last_leaves = IMT_Tree::read_tree(&last_superproof.superproof_leaves_path.unwrap())?;
    } else {
        let (zero_leaves, zero_root) = get_init_tree_data(IMT_DEPTH as u8);
        last_root = zero_root;
        last_leaves = IMT_Tree{ leafs: zero_leaves };
    }

    // Read aggregator_circuit_pkey and aggregator_circuit_vkey from file
    let aggregator_pkey_path = get_aggregation_circuit_proving_key_path(&config.aggregated_circuit_data);
    let aggregator_vkey_path = get_aggregation_circuit_vkey_path(&config.aggregated_circuit_data);
    let aggregator_circuit_pkey = read_bytes_from_file(&aggregator_pkey_path)?;
    let aggregator_circuit_vkey = GnarkGroth16Vkey::read_vk(&aggregator_vkey_path)?;


    let aggregation_result = QuantumV2CircuitInteractor::generate_aggregated_proof(
        reduced_proofs, 
        reduced_pis_vec, 
        reduced_circuit_vkeys, 
        last_root, 
        last_leaves.leafs, 
        aggregator_circuit_pkey, 
        aggregator_circuit_vkey
    );

    println!("aggregation_result {:?}", aggregation_result);

    Ok(())
}