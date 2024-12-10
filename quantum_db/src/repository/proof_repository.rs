use quantum_types::{enums::{proof_status::ProofStatus, proving_schemes::ProvingSchemes}, types::db::proof::Proof};
use sqlx::{any::AnyRow, Any, Execute, Pool, Row};
use quantum_utils::error_line;
use anyhow::{anyhow, Error, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;

pub async fn get_aggregation_waiting_proof_num(pool: &Pool<Any>) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("SELECT Count(*) as reduced_proof_count from proof where proof_status = ?")
                .bind(ProofStatus::Reduced.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}",ProofStatus::Reduced.as_u8());

    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) =>{
            let id: u64 = t.try_get_unchecked::<i64, &str>("reduced_proof_count")?.try_into()?;
            Ok(id)
        }
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
}

pub async fn insert_proof(pool: &Pool<Any>, proof_hash: &str, pis_path: &str, proof_path: &str, proof_status: ProofStatus, user_circuit_hash: &str, pis_json_string: &str)-> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into proof(proof_hash, pis_path, proof_path, proof_status, user_circuit_hash, public_inputs) VALUES(?,?,?,?,?,?)")
                .bind(proof_hash).bind(pis_path).bind(proof_path).bind(proof_status.as_u8() as i64).bind(user_circuit_hash).bind(pis_json_string);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}, {}, {}", proof_hash, pis_path, proof_path, proof_status.as_u8(), user_circuit_hash, pis_json_string);

    let row_affected = match query.execute(pool).await {
        Ok(t) => t.last_insert_id().map(|id| id as u64).ok_or_else(|| anyhow!("last insert id not present")),
        Err(e) =>Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_latest_proof_by_circuit_hash(pool: &Pool<Any>, circuit_hash: &str) -> AnyhowResult<Proof> {
    let query  = sqlx::query("SELECT * from proof where user_circuit_hash = ? order by id desc LIMIT 1").bind(circuit_hash);

    info!("{}", query.sql());
    info!("arguments: {}", circuit_hash);

    let proof = match query.fetch_one(pool).await{
        Ok(t) => get_proof_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    proof
}

pub async fn get_proof_by_proof_id(pool: &Pool<Any>, proof_id: u64) -> AnyhowResult<Proof> {
    let query  = sqlx::query("SELECT * from proof where id = ?")
                .bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}", proof_id);

    let proof = match query.fetch_one(pool).await{
        Ok(t) => get_proof_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    proof
}

pub async fn get_proof_by_proof_hash(pool: &Pool<Any>, proof_hash: &str) -> AnyhowResult<Proof> {
    let query  = sqlx::query("SELECT * from proof where proof_hash = ? order by id desc limit 1")
        .bind(proof_hash);

    info!("{}", query.sql());
    info!("arguments: {}", proof_hash);

    let proof = match query.fetch_one(pool).await{
        Ok(t) => get_proof_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    proof
}

// pub async fn get_latest_proofs_by_circuit_hash(pool: &Pool<MySql>, circuit_hash: Vec<String>, limit: u8) -> AnyhowResult<Proof> {
//     let query  = sqlx::query("SELECT * from proof where user_circuit_hash in (?) limit ?")
//         .bind(circuit_hash).bind(limit);
//
//     info!("{}", query.sql());
//     info!("arguments: {:?}, {}",circuit_hash, limit);
//
//     let proof = match query.fetch_one(pool).await{
//         Ok(t) => get_proof_from_mysql_row(&t),
//         Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
//     };
//     proof
// }

pub async fn get_proofs_in_superproof_id(pool: &Pool<Any>, superproof_id: u64) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("SELECT * from proof where superproof_id = ?")
                .bind(superproof_id as i64);

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

pub async fn update_proof_status(pool: &Pool<Any>, proof_id: u64, proof_status: ProofStatus) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE proof set proof_status = ? where id = ?")
                .bind(proof_status.as_u8() as i64).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", proof_status.as_u8(), proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_reduction_data(pool: &Pool<Any>, proof_id: u64, reducded_proof_receipt_path: &str, reduction_time: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set reducded_proof_receipt_path = ?, reduction_time = ?  where id = ?")
                .bind(reducded_proof_receipt_path).bind(reduction_time as i64).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}", reducded_proof_receipt_path, reduction_time, proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_id_in_proof(pool: &Pool<Any>, proof_id: u64, superproof_id: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set superproof_id = ? where id = ?")
                .bind(superproof_id as i64).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_id, proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_session_id_in_proof(pool: &Pool<Any>, proof_id: u64, session_id: &str) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set session_id = ? where id = ?")
                .bind(session_id).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", session_id, proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_cycle_used_in_proof(pool: &Pool<Any>, proof_id: u64, cycle_used: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE proof set cycle_used = ? where id = ?")
                .bind(cycle_used as i64).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", cycle_used, proof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_reduced_proofs_r0(pool: &Pool<Any>) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("
        SELECT * from proof join user_circuit_data on proof.user_circuit_hash = user_circuit_data.circuit_hash where proof.proof_status = ? and user_circuit_data.proving_scheme != ? order by id desc;
    ").bind(ProofStatus::Reduced.as_u8() as i64).bind(ProvingSchemes::Sp1.to_string());

    info!("{}", query.sql());
    info!("arguments: {}", ProofStatus::Reduced.as_u8());

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

pub async fn get_reduced_proofs_sp1(pool: &Pool<Any>) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("
        SELECT * from proof join user_circuit_data on proof.user_circuit_hash = user_circuit_data.circuit_hash where proof.proof_status = ? and user_circuit_data.proving_scheme = ? order by id desc;
    ").bind(ProofStatus::Reduced.as_u8() as i64).bind(ProvingSchemes::Sp1.to_string());

    info!("{}", query.sql());
    info!("arguments: {}", ProofStatus::Reduced.as_u8());

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


pub async fn get_reduced_proofs(pool: &Pool<Any>) -> AnyhowResult<Vec<Proof>> {
    let query  = sqlx::query("SELECT * from proof where proof_status = ? order by id")
                .bind(ProofStatus::Reduced.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}", ProofStatus::Reduced.as_u8());

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

fn get_proof_from_mysql_row(row: &AnyRow) -> AnyhowResult<Proof>{
    let proof_status_as_u8: u8 = row.try_get_unchecked::<i64, &str>("proof_status")?.try_into()?;
    let proof_status =  ProofStatus::from(proof_status_as_u8);
    let proof = Proof {
        id: row.try_get_unchecked::<Option<i64>, &str>("id")?.map(|id| id as u64),
        proof_hash: row.try_get_unchecked("proof_hash")?,
        pis_path: row.try_get_unchecked("pis_path")?,
        proof_path: row.try_get_unchecked("proof_path")?,
        superproof_id: row.try_get_unchecked::<Option<i64>, &str>("superproof_id")?.map(|id| id as u64),
        reduction_time: row.try_get_unchecked::<Option<i64>, &str>("reduction_time")?.map(|id| id as u64),
        proof_status: proof_status,
        user_circuit_hash: row.try_get_unchecked("user_circuit_hash")?,
        input_id: row.try_get_unchecked("input_id")?,
        session_id: row.try_get_unchecked("session_id")?,
        cycle_used: row.try_get_unchecked::<Option<i64>, &str>("cycle_used")?.map(|id| id as u64),
    };
    Ok(proof)
}

