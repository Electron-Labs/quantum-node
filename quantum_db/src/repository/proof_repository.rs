use quantum_types::enums::proof_status::ProofStatus;
use sqlx::{MySql, Pool, Row};

use anyhow::{anyhow, Error, Result as AnyhowResult};

use crate::error::error::CustomError;

pub async fn get_aggregation_waiting_proof_num(pool: &Pool<MySql>) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("SELECT Count(*) as reduced_proof_count from proof where proof_status = ?")
                .bind(ProofStatus::Reduced.as_u8());

    // info!("{}", query.sql());
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) =>{ 
            let id: u64 = t.try_get_unchecked("reduced_proof_count")?;
            Ok(id)
        }
        Err(e) => Err(anyhow!(CustomError::DB(e.to_string())))
    };
    reduction_circuit
}

pub async fn insert_proof(pool: &Pool<MySql>, proof_hash: &str, pis_path: &str, proof_path: &str, proof_status: ProofStatus, user_circuit_hash: &str)-> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into proof(proof_hash, pis_path, proof_path, proof_status, user_circuit_hash) VALUES(?,?,?,?,?)")
                .bind(proof_hash).bind(pis_path).bind(proof_path).bind(proof_status.as_u8()).bind(user_circuit_hash);

    // info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) =>Err(anyhow!(CustomError::DB(e.to_string())))
    };
    row_affected
}