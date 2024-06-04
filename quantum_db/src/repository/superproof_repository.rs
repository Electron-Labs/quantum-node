use std::task::Poll;

use quantum_types::{enums::superproof_status::SuperproofStatus, types::db::superproof::Superproof};
use sqlx::{mysql::MySqlRow, Error, Execute, MySql, Pool, Row};
use anyhow::{anyhow, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;

pub async fn get_superproof_by_id(pool: &Pool<MySql>, id: u64) -> AnyhowResult<Superproof> {
    let query  = sqlx::query("SELECT * from superproof where id = ?")
                .bind(id);

    info!("{}", query.sql());
    let superproof = match query.fetch_one(pool).await{
        Ok(t) => get_superproof_from_row(t),
        Err(e) => {
            info!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(e.to_string())))
        }
    };
    superproof
}

pub async fn insert_new_superproof(pool: &Pool<MySql>, proof_ids_string: &str, superproof_status: SuperproofStatus) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into superproof(proof_ids, status) VALUES(?,?)")
        .bind(proof_ids_string).bind(superproof_status.as_u8());

    info!("{}", query.sql());
    let superproof_id = match query.execute(pool).await {
        Ok(t) => Ok(t.last_insert_id()),
        Err(e) => Err(e)
    };
    superproof_id
}

pub async fn update_superproof_status(pool: &Pool<MySql>, superproof_status: SuperproofStatus, superproof_id: u64) -> AnyhowResult<()>{
    let query  = sqlx::query("UPDATE superproof set status = ? where id = ?")
                .bind(superproof_status.as_u8()).bind(superproof_id);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(e.to_string())))
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
        superproof_pis_path: row.try_get_unchecked("superproof_pis_path")?,
        transaction_hash: row.try_get_unchecked("transaction_hash")?,
        gas_cost: row.try_get_unchecked("gas_cost")?,
        agg_time: row.try_get_unchecked("agg_time")?,
        status: superproof_status,
        superproof_root: row.try_get_unchecked("superproof_root")?,
        superproof_leaves_path: row.try_get_unchecked("superproof_leaves_path")?
    };

    Ok(superproof)
}