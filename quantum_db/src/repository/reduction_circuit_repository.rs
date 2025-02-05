// use quantum_types::{enums::proving_schemes::ProvingSchemes, types::db::reduction_circuit::ReductionCircuit};
// use quantum_utils::error_line;
// use sqlx::{mysql::MySqlRow, Error, MySql, Pool, Row, Execute};

// use anyhow::{anyhow, Result as AnyhowResult};
// use tracing::info;
// use std::str::FromStr;

// use crate::error::error::CustomError;

// pub async fn get_reduction_circuit_by_n_inner_commitments(pool: &Pool<MySql>, n_inner_commitments: u8) -> AnyhowResult<ReductionCircuit>{
//     let query  = sqlx::query("SELECT * from reduction_circuit where n_inner_commitments = ?")
//                 .bind(n_inner_commitments);

//     info!("{}", query.sql());
//     info!("arguments: {}", n_inner_commitments);

//     let reduction_circuit = match query.fetch_one(pool).await{
//         Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
//         Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
//     };
//     reduction_circuit
// }

// pub async fn get_reduction_circuit_for_user_circuit(pool: &Pool<MySql>, user_circuit_hash: &str) -> AnyhowResult<ReductionCircuit> {
//     let query  = sqlx::query("SELECT * from reduction_circuit where circuit_id = (select reduction_circuit_id from user_circuit_data where circuit_hash = ?)")
//                 .bind(user_circuit_hash);

//     info!("{}", query.sql());
//     info!("arguments: {}", user_circuit_hash);

//     let reduction_circuit = match query.fetch_one(pool).await{
//         Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
//         Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
//     };
//     reduction_circuit
// }
// // applies only for circom and gnark circuits
// pub async fn check_if_n_inner_commitments_compatible_reduction_circuit_id_exist(pool: &Pool<MySql>, n_inner_commitments: u8) -> Option<ReductionCircuit>{
//     let rc = get_reduction_circuit_by_n_inner_commitments(pool, n_inner_commitments).await;
//     match rc {
//         Ok(rc) => Some(rc),
//         Err(_) => None
//     }
// }

// fn get_reduction_circuit_data_from_mysql_row(row: MySqlRow) -> AnyhowResult<ReductionCircuit>{
//     let proving_scheme = match ProvingSchemes::from_str(row.try_get_unchecked("proving_scheme")?) {
//         Ok(ps) => Ok(ps),
//         Err(e) => Err(anyhow!(CustomError::DB(e)))
//     };
//     let reduction_circuit = ReductionCircuit {
//         circuit_id: row.try_get_unchecked("circuit_id")?,
//         proving_key_path: row.try_get_unchecked("proving_key_path")?,
//         vk_path: row.try_get_unchecked("vk_path")?,
//         n_inner_pis: row.try_get_unchecked("n_inner_pis")?,
//         n_inner_commitments: row.try_get_unchecked("n_inner_commitments")?,
//         proving_scheme: proving_scheme?
//     };
//     Ok(reduction_circuit)
// }

// // Sending ReductionCircuit type with reduction_circuit.id = None, return id
// pub async fn add_reduction_circuit_row(pool: &Pool<MySql>, reduction_circuit: ReductionCircuit) -> AnyhowResult<u64, Error> {
//     let query  = sqlx::query("INSERT into reduction_circuit(circuit_id, proving_key_path, vk_path, n_inner_pis, n_inner_commitments, proving_scheme) VALUES(?,?,?,?,?,?)")
//                 .bind(reduction_circuit.circuit_id.clone()).bind(reduction_circuit.proving_key_path.clone()).bind(reduction_circuit.vk_path.clone()).bind(reduction_circuit.n_inner_pis).bind(reduction_circuit.n_inner_commitments).bind(reduction_circuit.proving_scheme.to_string());

//     info!("{}", query.sql());
//     info!("arguments: {}, {}, {}, {}, {:?}, {}", reduction_circuit.circuit_id, reduction_circuit.proving_key_path, reduction_circuit.vk_path, reduction_circuit.n_inner_pis, reduction_circuit.n_inner_commitments, reduction_circuit.proving_scheme.to_string());

//     let row_affected = match query.execute(pool).await {
//         Ok(t) => Ok(t.rows_affected()),
//         Err(e) => Err(e)
//     };
//     row_affected
// }

// // get ReductionCircuit data from reduction_circuit_id
// pub async fn get_reduction_circuit_data_by_id(pool: &Pool<MySql>, id: &str) -> AnyhowResult<ReductionCircuit> {
//     let query  = sqlx::query("SELECT * from reduction_circuit where circuit_id = ?")
//                 .bind(id);

//     info!("{}", query.sql());
//     info!("arguments: {}", id);

//     let reduction_circuit = match query.fetch_one(pool).await{
//         Ok(t) => get_reduction_circuit_data_from_mysql_row(t),
//         Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
//     };
//     reduction_circuit
// }