use chrono::NaiveDateTime;
use quantum_types::{enums::superproof_status::SuperproofStatus, types::db::superproof::Superproof};
use quantum_utils::error_line;
use sqlx::{any::AnyRow, Any, Execute, Pool, Row};
use anyhow::{anyhow, Error as AnyhowError, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;

pub async fn get_superproof_by_id(pool: &Pool<Any>, id: u64) -> AnyhowResult<Superproof> {
    let query  = sqlx::query("SELECT * from superproof where id = ?")
                .bind(id as i64);

    info!("{}", query.sql());
    info!("arguments: {}", id);

    let superproof = match query.fetch_one(pool).await{
        Ok(t) => get_superproof_from_row(t).map_err(|err| anyhow!(error_line!(err))),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    superproof
}

pub async fn insert_new_superproof(pool: &Pool<Any>, proof_ids_string: &str, superproof_status: SuperproofStatus) -> AnyhowResult<u64, AnyhowError> {
    let query  = sqlx::query("INSERT into superproof(proof_ids, status) VALUES(?,?)")
        .bind(proof_ids_string).bind(superproof_status.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", proof_ids_string, superproof_status.as_u8());

    let superproof_id = match query.execute(pool).await {
        Ok(t) => t.last_insert_id().map(|id| id as u64).ok_or_else(|| anyhow!("last insert id not present")),
        Err(e) => Err(anyhow!(error_line!(e)))
    };
    superproof_id
}

pub async fn update_superproof_status(pool: &Pool<Any>, superproof_status: SuperproofStatus, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set status = ? where id = ?")
                .bind(superproof_status.as_u8() as i64).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_status.as_u8(), superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_r0_leaves_path(pool: &Pool<Any>, r0_leaves_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set r0_leaves_path = ? where id = ?")
                .bind(r0_leaves_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", r0_leaves_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_sp1_leaves_path(pool: &Pool<Any>, sp1_leaves_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set sp1_leaves_path = ? where id = ?")
                .bind(sp1_leaves_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", sp1_leaves_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_imt_proof_path(pool: &Pool<Any>, imt_proof_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set imt_proof_path = ? where id = ?")
                .bind(imt_proof_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", imt_proof_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_imt_pis_path(pool: &Pool<Any>, imt_pis_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set imt_pis_path = ? where id = ?")
                .bind(imt_pis_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", imt_pis_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_previous_superproof_root(pool: &Pool<Any>, previous_superproof_root: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set previous_superproof_root = ? where id = ?")
                .bind(previous_superproof_root).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", previous_superproof_root, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_r0_root(pool: &Pool<Any>, r0_root: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set r0_root = ? where id = ?")
                .bind(r0_root).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", r0_root, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_sp1_root(pool: &Pool<Any>, sp1_root: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set sp1_root = ? where id = ?")
                .bind(sp1_root).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", sp1_root, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}



pub async fn update_superproof_root(pool: &Pool<Any>, superproof_root: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_root = ? where id = ?")
                .bind(superproof_root).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_root, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_cycles_in_superproof(pool: &Pool<Any>, agg_cycle: u64, total_cycle_used: u64, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set agg_cycle_used = ?, total_cycle_used =?  where id = ?")
                .bind(agg_cycle as i64).bind(total_cycle_used as i64).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}", agg_cycle, total_cycle_used, superproof_id );

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_agg_time(pool: &Pool<Any>, agg_time: u64, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set agg_time = ? where id = ?")
                .bind(agg_time as i64).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", agg_time, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_gas_cost(pool: &Pool<Any>, gas_cost: f64, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set gas_cost = ? where id = ?")
                .bind(gas_cost).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", gas_cost, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}


pub async fn update_transaction_hash(pool: &Pool<Any>, transaction_hash: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set transaction_hash = ? where id = ?")
                .bind(transaction_hash).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", transaction_hash, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_proof_path(pool: &Pool<Any>, superproof_proof_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_proof_path = ? where id = ?")
                .bind(superproof_proof_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_proof_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_pis_path(pool: &Pool<Any>, superproof_pis_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_pis_path = ? where id = ?")
                .bind(superproof_pis_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", superproof_pis_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_r0_receipts_path(pool: &Pool<Any>, r0_receipt_path: &str, r0_snark_receipt_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set r0_receipt_path = ?, r0_snark_receipt_path = ? where id = ?")
                .bind(r0_receipt_path).bind(r0_snark_receipt_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}", r0_receipt_path, r0_snark_receipt_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_sp1_snark_receipt_path(pool: &Pool<Any>, sp1_snark_receipt_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set sp1_snark_receipt_path = ? where id = ?")
                .bind(sp1_snark_receipt_path).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", sp1_snark_receipt_path, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_onchain_submission_time(pool: &Pool<Any>, onchain_submission_time: NaiveDateTime, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set onchain_submission_time = ? where id = ?")
                .bind(onchain_submission_time).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", onchain_submission_time, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_total_proving_time(pool: &Pool<Any>, total_proving_time: u64, superproof_id: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE superproof set total_proving_time = ? where id = ?")
                .bind(total_proving_time as i64).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", total_proving_time, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_session_id_superproof(pool: &Pool<Any>, session_id: &str, superproof_id: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE superproof set session_id = ? where id = ?")
                .bind(session_id).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", session_id, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_snark_session_id_superproof(pool: &Pool<Any>, snark_session_id: &str, superproof_id: u64) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE superproof set snark_session_id = ? where id = ?")
                .bind(snark_session_id).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", snark_session_id, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}



fn get_superproof_from_row(row: AnyRow) -> AnyhowResult<Superproof> {
    let superproof_status_as_u8: u8 = row.try_get_unchecked::<i64, &str>("status")?.try_into()?;
    let superproof_status =  SuperproofStatus::from(superproof_status_as_u8);
    let superproof = Superproof {
        id: row.try_get_unchecked::<Option<i64>, &str>("id")?.map(|t| t as u64),
        proof_ids: row.try_get_unchecked("proof_ids")?,
        superproof_proof_path: row.try_get_unchecked("superproof_proof_path")?,
        transaction_hash: row.try_get_unchecked("transaction_hash")?,
        agg_time: row.try_get_unchecked::<Option<i64>, &str>("agg_time")?.map(|t| t as u64),
        status: superproof_status,
        superproof_root: row.try_get_unchecked("superproof_root")?,
        superproof_leaves_path: row.try_get_unchecked("superproof_leaves_path")?,
        r0_leaves_path: row.try_get_unchecked("r0_leaves_path")?,
        sp1_leaves_path: row.try_get_unchecked("sp1_leaves_path")?,
        onchain_submission_time: row.try_get_unchecked("onchain_submission_time")?,
        previous_superproof_root: row.try_get_unchecked("previous_superproof_root")?,
        imt_proof_path: row.try_get_unchecked("imt_proof_path")?,
        imt_pis_path: row.try_get_unchecked("imt_pis_path")?,
        r0_root: row.try_get_unchecked("r0_root")?,
        sp1_root: row.try_get_unchecked("sp1_root")?,
        r0_snark_receipt_path: row.try_get_unchecked("r0_snark_receipt_path")?,
    };

    Ok(superproof)
}

pub async fn get_last_aggregated_superproof(pool: &Pool<Any>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id desc LIMIT 1")
                                                    .bind(SuperproofStatus::ProvingDone.as_u8() as i64);

    info!("{}", query.sql());
    let superproof = match query.fetch_optional(pool).await{
        Ok(t) => Ok(t),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    let superproof = superproof?;

    let superproof = match superproof {
        Some(t) => Some(get_superproof_from_row(t)?),
        None =>  None,
    };
    Ok(superproof)
}

pub async fn get_last_verified_superproof(pool: &Pool<Any>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id desc LIMIT 1")
                                                    .bind(SuperproofStatus::SubmittedOnchain.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}", SuperproofStatus::SubmittedOnchain.as_u8());

    let superproof = match query.fetch_optional(pool).await{
        Ok(t) => Ok(t),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    let superproof = superproof?;

    let superproof = match superproof {
        Some(t) => Some(get_superproof_from_row(t)?),
        None =>  None,
    };
    Ok(superproof)
}

pub async fn get_first_non_submitted_superproof(pool: &Pool<Any>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id LIMIT 1")
                                                    .bind(SuperproofStatus::ProvingDone.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}", SuperproofStatus::ProvingDone.as_u8());

    let superproof = match query.fetch_optional(pool).await{
        Ok(t) => Ok(t),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    let superproof = superproof?;

    let superproof = match superproof {
        Some(t) => Some(get_superproof_from_row(t)?),
        None =>  None,
    };
    Ok(superproof)
}

pub async fn update_superproof_fields_after_onchain_submission(pool: &Pool<Any>, transaction_hash: &str, status: SuperproofStatus, gas_used: u64, superproof_id: u64) -> AnyhowResult<()> {
    let query = sqlx::query("UPDATE superproof SET transaction_hash = ?, status = ?, total_proof_ver_cost = ? WHERE id = ?")
            .bind(transaction_hash).bind(status.as_u8() as i64).bind(gas_used as i64).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}", transaction_hash, status.as_u8(), gas_used, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_gas_data(pool: &Pool<Any>, gas_cost: f64, eth_price: f64, total_cost_usd: f64, superproof_id: u64) -> AnyhowResult<()> {
    let query = sqlx::query("UPDATE superproof SET gas_cost = ?, eth_price = ?, total_cost_usd = ? WHERE id = ?")
            .bind(gas_cost).bind(eth_price).bind(total_cost_usd).bind(superproof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}", gas_cost, eth_price, total_cost_usd, superproof_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}