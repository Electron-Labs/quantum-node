use quantum_types::{enums::proof_status::ProofStatus, types::db::proof::Proof};
use sqlx::{mysql::MySqlRow, Execute, MySql, Pool, Row};
use quantum_utils::error_line;
use anyhow::{anyhow, Error, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;

pub async fn get_aggregation_waiting_proof_num(pool: &Pool<MySql>) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("SELECT Count(*) as reduced_proof_count from proof where proof_status = ?")
                .bind(ProofStatus::Reduced.as_u8());

    info!("{}", query.sql());
    info!("arguments: {}",ProofStatus::Reduced.as_u8());
    
    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) =>{ 
            let id: u64 = t.try_get_unchecked("reduced_proof_count")?;
            Ok(id)
        }
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
}

pub async fn insert_proof(pool: &Pool<MySql>, proof_hash: &str, pis_path: &str, proof_path: &str, proof_status: ProofStatus, user_circuit_hash: &str)-> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into proof(proof_hash, pis_path, proof_path, proof_status, user_circuit_hash) VALUES(?,?,?,?,?)")
                .bind(proof_hash).bind(pis_path).bind(proof_path).bind(proof_status.as_u8()).bind(user_circuit_hash);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}, {}", proof_hash, pis_path, proof_path, proof_status.as_u8(), user_circuit_hash);

    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) =>Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_proof_by_proof_hash(pool: &Pool<MySql>, proof_hash: &str) -> AnyhowResult<Proof> {
    let query  = sqlx::query("SELECT * from proof where proof_hash = ?")
                .bind(proof_hash);

    info!("{}", query.sql());
    info!("arguments: {}", proof_hash);
    
    let proof = match query.fetch_one(pool).await{
        Ok(t) => get_proof_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    proof
}

pub async fn get_proofs_in_superproof_id(pool: &Pool<MySql>, superproof_id: u64) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("SELECT * from proof where superproof_id = ?")
                .bind(superproof_id);

    info!("{}", query.sql());
    info!("arguments: {}", superproof_id);

    let rows = match query.fetch_all(pool).await{
        Ok(rows) => Ok(rows),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    }?;
    let mut proofs = vec![];
    for row in rows {
        let proof = get_proof_from_mysql_row(&row)?;
        proofs.push(proof);
    }

    return Ok(proofs)
}

pub async fn update_proof_status(pool: &Pool<MySql>, proof_hash: &str, proof_status: ProofStatus) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE proof set proof_status = ? where proof_hash = ?")
                .bind(proof_status.as_u8()).bind(proof_hash);

    info!("{}", query.sql());
    info!("arguments: {}, {}", proof_status.as_u8(), proof_hash);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_reduction_data(pool: &Pool<MySql>, proof_id: &str, reduction_proof_path: &str, reduction_pis_path: &str, reduction_time: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set reduction_proof_path = ?, reduction_proof_pis_path = ?, reduction_time = ?  where proof_hash = ?")
                .bind(reduction_proof_path).bind(reduction_pis_path).bind(reduction_time).bind(proof_id);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}", reduction_proof_path, reduction_pis_path, reduction_time, proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_id_in_proof(pool: &Pool<MySql>, proof_hash: &str, superproof_id: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set superproof_id = ? where proof_hash = ?")
                .bind(superproof_id).bind(proof_hash);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_id, proof_hash);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_n_reduced_proofs(pool: &Pool<MySql>, n: u64) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("SELECT * from proof where proof_status = ? order by id LIMIT ?")
                .bind(ProofStatus::Reduced.as_u8()).bind(n);

    info!("{}", query.sql());
    info!("arguments: {}, {}", ProofStatus::Reduced.as_u8(), n);

    let db_rows = match query.fetch_all(pool).await {
        Ok(t) => Ok(t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))  
    };

    let db_rows = db_rows?;
    let mut proofs: Vec<Proof> = vec![];
    for row in db_rows.iter()  {
        proofs.push(get_proof_from_mysql_row(row)?);
    }
    Ok(proofs)
}

fn get_proof_from_mysql_row(row: &MySqlRow) -> AnyhowResult<Proof>{
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

