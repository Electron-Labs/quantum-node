use quantum_db::repository::reduction_circuit_repository::get_reduction_circuit_for_user_circuit;
use quantum_types::types::{aggregator::{AggregatorCircuitData, IMTLeaves, InnerCircuitData}, db::proof::Proof};
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};

pub async fn handle_aggregation(pool: &Pool<MySql>, proofs: Vec<Proof>) -> AnyhowResult<()> {
    // 1. Create Vec<GnarkVerifier>, GnarkVerifier {ReducedGnarkProof, ReducedGnarkVK, ReducedPIS}
    println!("Starting to create inner circuits data");
    let mut inner_circuit_data_vec: Vec<InnerCircuitData> = Vec::new();
    for proof in proofs {
        let reduction_proof_path = proof.reduction_proof_path.clone().unwrap();// TODO: Replace these unwraps with proper checks
        let reduction_proof_pis_path = proof.reduction_proof_pis_path.clone().unwrap();
        let reduction_circuit = get_reduction_circuit_for_user_circuit(pool, &proof.user_circuit_hash).await?;
        let inner_circuit_data = InnerCircuitData::construct_from_paths(&reduction_proof_path, &reduction_proof_pis_path, &reduction_circuit.vk_path)?;
        inner_circuit_data_vec.push(inner_circuit_data);
    }

    // 2. Read agg_circuit pk_bytes
    // 3. Read agg_circuit vk_bytes
    let aggregation_circuit_pk_path = "";
    let aggregation_circuit_vk_path = "";
    println!("Loading up aggregator circuit data");
    let aggregator_circuit_data = AggregatorCircuitData::read_data(aggregation_circuit_pk_path, aggregation_circuit_vk_path)?;

    // 4. Extract CurLeaves
    let imt_cur_leaves: Vec<u8> = Vec::new();
    let imt_curr_leaves = IMTLeaves::deserialize(&imt_cur_leaves)?;

    

    Ok(())
}