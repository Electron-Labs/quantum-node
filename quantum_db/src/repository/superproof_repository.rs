use chrono::NaiveDateTime;
use quantum_types::{enums::superproof_status::SuperproofStatus, types::db::superproof::Superproof};
use quantum_utils::error_line;
use sqlx::{mysql::MySqlRow , Execute, MySql, Pool, Row};
use anyhow::{anyhow, Error as AnyhowError, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;

pub async fn get_superproof_by_id(pool: &Pool<MySql>, id: u64) -> AnyhowResult<Superproof> {
    let query  = sqlx::query("SELECT * from superproof where id = ?")
                .bind(id);

    info!("{}", query.sql());
    let superproof = match query.fetch_one(pool).await{
        Ok(t) => get_superproof_from_row(t).map_err(|err| anyhow!(error_line!(err))),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    superproof
}

pub async fn insert_new_superproof(pool: &Pool<MySql>, proof_ids_string: &str, superproof_status: SuperproofStatus) -> AnyhowResult<u64, AnyhowError> {
    let query  = sqlx::query("INSERT into superproof(proof_ids, status) VALUES(?,?)")
        .bind(proof_ids_string).bind(superproof_status.as_u8());

    info!("{}", query.sql());
    let superproof_id = match query.execute(pool).await {
        Ok(t) => Ok(t.last_insert_id()),
        Err(e) => Err(anyhow!(error_line!(e)))
    };
    superproof_id
}

pub async fn update_superproof_status(pool: &Pool<MySql>, superproof_status: SuperproofStatus, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set status = ? where id = ?")
                .bind(superproof_status.as_u8()).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_leaves_path(pool: &Pool<MySql>, superproof_leaves_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_leaves_path = ? where id = ?")
                .bind(superproof_leaves_path).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_root(pool: &Pool<MySql>, superproof_root: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_root = ? where id = ?")
                .bind(superproof_root).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_agg_time(pool: &Pool<MySql>, agg_time: u64, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set agg_time = ? where id = ?")
                .bind(agg_time).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_gas_cost(pool: &Pool<MySql>, gas_cost: f64, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set gas_cost = ? where id = ?")
                .bind(gas_cost).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}


pub async fn update_transaction_hash(pool: &Pool<MySql>, transaction_hash: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set transaction_hash = ? where id = ?")
                .bind(transaction_hash).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_proof_path(pool: &Pool<MySql>, superproof_proof_path: &str, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set superproof_proof_path = ? where id = ?")
                .bind(superproof_proof_path).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn update_superproof_onchain_submission_time(pool: &Pool<MySql>, onchain_submission_time: NaiveDateTime, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set onchain_submission_time = ? where id = ?")
                .bind(onchain_submission_time).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

fn get_superproof_from_row(row: MySqlRow) -> AnyhowResult<Superproof> {
    let superproof_status_as_u8: u8 = row.try_get_unchecked("status")?;
    let superproof_status =  SuperproofStatus::from(superproof_status_as_u8);
    let superproof = Superproof {
        id: row.try_get_unchecked("id")?,
        proof_ids: row.try_get_unchecked("proof_ids")?,
        superproof_proof_path: row.try_get_unchecked("superproof_proof_path")?,
        transaction_hash: row.try_get_unchecked("transaction_hash")?,
        gas_cost: row.try_get("gas_cost")?,
        agg_time: row.try_get_unchecked("agg_time")?,
        status: superproof_status,
        superproof_root: row.try_get_unchecked("superproof_root")?,
        superproof_leaves_path: row.try_get_unchecked("superproof_leaves_path")?,
        onchain_submission_time: row.try_get_unchecked("onchain_submission_time")?,
        eth_price: row.try_get_unchecked("eth_price")?,
    };

    Ok(superproof)
}


pub async fn get_last_superproof(pool: &Pool<MySql>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof order by id desc LIMIT 1 OFFSET 1");

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

pub async fn get_last_aggregated_superproof(pool: &Pool<MySql>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id desc LIMIT 1")
                                                    .bind(SuperproofStatus::ProvingDone.as_u8());

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

pub async fn get_last_verified_superproof(pool: &Pool<MySql>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id desc LIMIT 1")
                                                    .bind(SuperproofStatus::SubmittedOnchain.as_u8());

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

pub async fn get_first_non_submitted_superproof(pool: &Pool<MySql>) -> AnyhowResult<Option<Superproof>> {
    let query  = sqlx::query("SELECT * from superproof where status = ? order by id LIMIT 1")
                                                    .bind(SuperproofStatus::ProvingDone.as_u8());

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

pub async fn update_superproof_fields_after_onchain_submission(pool: &Pool<MySql>, transaction_hash: &str, gas_cost: f64, eth_price: f64, status: SuperproofStatus, superproof_id: u64) -> AnyhowResult<()> {
    let query = sqlx::query("UPDATE superproof SET transaction_hash = ?, status = ?, gas_cost = ?, eth_price = ? WHERE id = ?")
            .bind(transaction_hash).bind(status.as_u8()).bind(gas_cost).bind(eth_price).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}