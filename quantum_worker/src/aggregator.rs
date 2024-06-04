use quantum_db::repository::reduction_circuit_repository::get_reduction_circuit_for_user_circuit;
use quantum_types::types::{aggregator::{AggregatorCircuitData, IMTLeaves, InnerCircuitData}, db::proof::Proof};
use sqlx::{MySql, Pool};
use anyhow::{Ok, Result as AnyhowResult};

pub async fn handle_aggregation(pool: &Pool<MySql>, proofs: Vec<Proof>) -> AnyhowResult<()> {
    
    Ok(())
}