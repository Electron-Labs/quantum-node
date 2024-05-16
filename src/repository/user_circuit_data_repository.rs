use sqlx::mysql::MySqlRow;
use sqlx::{Execute, MySql, Row};

use crate::types::user_circuit_data;
use crate::{connection::get_pool, types::user_circuit_data::UserCircuitData};

pub async fn get_user_circuit_data(circuit_hash: &str) -> Option<UserCircuitData>{
    let pool = get_pool().await;
    let query  = sqlx::query("SELECT * from user_circuit_data where circuit_hash = ?")
                .bind(circuit_hash);

    // info!("{}", query.sql());
    let user_circuit_data = match query.fetch_optional(pool).await.unwrap() {
        Some(t) => Some(get_user_circuit_data_from_mysql_row(t)),
        None => None,
    };
    user_circuit_data
}

pub async fn insert_user_circuit_data(circuit_hash: &str, vk_path: &str, reduction_circuit_id: Option<u64>, pis_len: u8) {
    let pool = get_pool().await;
    let query  = sqlx::query("INSERT into user_circuit_data(circuit_hash, vk_path, reduction_circuit_id, pis_len) VALUES(?,?,?,?)")
                .bind(circuit_hash).bind(vk_path).bind(reduction_circuit_id).bind(pis_len);

    // info!("{}", query.sql());
    let row_affected = query.execute(pool).await.unwrap();
}


fn get_user_circuit_data_from_mysql_row(row: MySqlRow) -> UserCircuitData{
    let user_circuit_data = UserCircuitData {
        circuit_hash : row.try_get_unchecked("circuit_hash").unwrap(),
        vk_path: row.try_get_unchecked("vk_path").unwrap(),
        reduction_circuit_id: row.try_get_unchecked("reduction_circuit_id").unwrap(),
        pis_len: row.try_get_unchecked("pis_len").unwrap(),
    };
    user_circuit_data
}