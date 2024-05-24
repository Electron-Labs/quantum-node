use quantum_types::types::db::superproof::Superproof;
use sqlx::{mysql::MySqlRow, MySql, Pool, Row};
use anyhow::{anyhow, Result as AnyhowResult};

use crate::error::error::CustomError;

pub async fn get_superproof_by_id(pool: &Pool<MySql>, id: u64) -> AnyhowResult<Superproof> {
    let query  = sqlx::query("SELECT * from superproof where id = ?")
                .bind(id);

    // info!("{}", query.sql());
    let superproof = match query.fetch_one(pool).await{
        Ok(t) => get_superproof_from_row(t),
        Err(e) => {
            println!("error in super proof fetch");
            Err(anyhow!(CustomError::DB(e.to_string())))
        }
    };
    superproof
}

fn get_superproof_from_row(row: MySqlRow) -> AnyhowResult<Superproof> {
    let superproof = Superproof {
        id: row.try_get_unchecked("id")?,
        proof_ids: row.try_get_unchecked("proof_ids")?,
        superproof_proof_path: row.try_get_unchecked("superproof_proof_path")?,
        superproof_pis_path: row.try_get_unchecked("superproof_pis_path")?,
        transaction_hash: row.try_get_unchecked("transaction_hash")?,
        gas_cost: row.try_get_unchecked("gas_cost")?,
        agg_time: row.try_get_unchecked("agg_time")?,
    };

    Ok(superproof)
}