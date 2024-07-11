use anyhow::{anyhow, Result as AnyhowResult};
use quantum_db::error::error::CustomError;
use quantum_utils::error_line;
use sqlx::{Execute, MySql, Pool};
use tracing::info;


pub async fn insert_dummy_data_superproof(pool: &Pool<MySql>, proof_ids: &str) -> AnyhowResult<()>{
    let query: sqlx::query::Query<MySql, _> = sqlx::query("INSERT INTO superproof (
    proof_ids, 
    superproof_proof_path, 
    superproof_pis_path, 
    transaction_hash, 
    gas_cost, 
    eth_price, 
    agg_time, 
    status, 
    superproof_root, 
    superproof_leaves_path, 
    onchain_submission_time
) VALUES (
    ?, 
    './storage/superproofs/3/proof.bin', 
    NULL, 
    '0x6d97cfe28477880a2ace08fd96aafad6885771bdd837951966c7e22bc49f6607', 
    3.233, 
    3527.170, 
    NULL, 
    3, 
    NULL, 
    NULL, 
    NOW()
)").bind(proof_ids);
    
    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn delete_dummy_data_superproof(pool: &Pool<MySql>, proof_ids: &str) -> AnyhowResult<()>{
    let query = sqlx::query("DELETE FROM superproof WHERE proof_ids=?").bind(proof_ids);
    
    info!("{}", query.sql());

    let row_affected = match query.execute(&mut *pool.acquire().await.unwrap()).await{
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}