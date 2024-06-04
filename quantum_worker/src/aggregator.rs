use quantum_db::repository::reduction_circuit_repository::get_reduction_circuit_for_user_circuit;
use quantum_types::types::{aggregator::{AggregatorCircuitData, IMTLeaves, InnerCircuitData}, db::proof::Proof, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}};
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};

pub async fn handle_aggregation(pool: &Pool<MySql>, proofs: Vec<Proof>, proof_ids: Vec<u64>, superproof_id: u64) -> AnyhowResult<()> {
    let reduced_proofs = Vec::<GnarkGroth16Proof>::new();
    let reduced_pis = Vec::<GnarkGroth16Pis>::new();
    let reduction_circuit_vkeys = Vec::<GnarkGroth16Vkey>::new();


    // 1. Extract reduced proof path from db corresponding to proof id
    // 2. Extract reduced pis path from db corresponding to proof id
    // 3. Extract reduced circuit vkeys path from db corresponding to proof id

    // TODO Utkarsh
    /*  
        if (superproof_id - 1) {
            extract Some(superproof_root, superproof_leaves_path)
        } else {
            None -> get zero_root and zero_leaves from quantum_circuits
        }
    */

    // Read aggregator_circuit_pkey and aggregator_circuit_vkey from file

    // fn generate_aggregated_proof(
    //     reduced_proofs: Vec<GnarkGroth16Proof>, 
    //     reduced_pis: Vec<GnarkGroth16Pis>, 
    //     reduction_circuit_vkeys: Vec<GnarkGroth16Vkey>, 
    //     old_root: KeccakHashOut,
    //     old_leaves: Vec<QuantumLeaf>,
    //     aggregator_circuit_pkey: Vec<u8>, 
    //     aggregator_circuit_vkey: GnarkGroth16Vkey
    // ) -> GenerateAggregatedProofResult;
    Ok(())
}