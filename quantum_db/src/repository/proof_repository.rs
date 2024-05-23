use quantum_types::{enums::proof_status::ProofStatus, types::db::proof::Proof};
use sqlx::{mysql::MySqlRow, MySql, Pool, Row};

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

pub async fn get_proof_by_proof_hash(pool: &Pool<MySql>, proof_hash: &str) -> AnyhowResult<Proof> {
    let query  = sqlx::query("SELECT * from proof where proof_hash = ?")
                .bind(proof_hash);

    // info!("{}", query.sql());
    let proof = match query.fetch_one(pool).await{
        Ok(t) => get_proof_from_mysql_row(t),
        Err(e) => Err(anyhow!(CustomError::DB(e.to_string())))
    };
    proof
}

fn get_proof_from_mysql_row(row: MySqlRow) -> AnyhowResult<Proof>{
    let proof_status_as_u8: u8 = row.try_get_unchecked("proof_status")?;
    let proof_status =  ProofStatus::from(proof_status_as_u8);
    let proof = Proof {
        id: row.try_get_unchecked("id")?,
        proof_hash: row.try_get_unchecked("proof_hash")?,
        pis_path: row.try_get_unchecked("pis_path")?,
        proof_path: row.try_get_unchecked("proof_path")?,
        reduction_proof_path: row.try_get_unchecked("reduction_proof_path")?,
        reduction_proof_pis_path: row.try_get_unchecked("reduction_proof_pis_path")?,
        superproof_id: row.try_get_unchecked("superproof_id")?,
        reduction_time: row.try_get_unchecked("reduction_time")?,
        proof_status: proof_status,
        user_circuit_hash: row.try_get_unchecked("user_circuit_hash")?,
    };
    Ok(proof)
}

