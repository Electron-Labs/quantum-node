use quantum_types::{enums::{task_status::TaskStatus, task_type::TaskType}, types::db::task::Task};
use quantum_utils::error_line;
use sqlx::{any::AnyRow, Any, Execute, Pool};
use sqlx::Row;
use anyhow::{anyhow, Error, Result as AnyhowResult};
use tracing::info;

use crate::error::error::CustomError;
// use crate::connection::get_pool;

pub async fn create_circuit_reduction_task(pool: &Pool<Any>,user_circuit_hash: &str, task_type: TaskType, task_status: TaskStatus) -> AnyhowResult<u64, Error>{
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, task_status) VALUES(?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8() as i64).bind(task_status.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}", user_circuit_hash, task_type.as_u8(), task_status.as_u8());

    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_unpicked_tasks(pool: &Pool<Any>, limit: u64) -> Result<Vec<Task>, Error> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    let query  = sqlx::query("SELECT * from task where task_status = ? order by id limit ?")
                .bind(TaskStatus::NotPicked.as_u8() as i64).bind(limit as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", TaskStatus::NotPicked.as_u8(), limit);

    let reduction_circuit = match query.fetch_all(pool).await{
        Ok(t) => {
            let mut rows = vec![];
            for row in t {
                rows.push(get_task_from_mysql_row(row)?);
            }
            Ok(rows)
        },
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
}

pub fn get_task_from_mysql_row(r: AnyRow) -> AnyhowResult<Task> {
    let task_type: u8 = r.try_get_unchecked::<i64, &str>("task_type")?.try_into()?;
    let task_status: u8 = r.try_get_unchecked::<i64, &str>("task_status")?.try_into()?;
    let task = Task{
        id: r.try_get_unchecked::<Option<i64>, &str>("id")?.map(|id| id as u64),
        user_circuit_hash: r.try_get_unchecked("user_circuit_hash")?,
        task_type: TaskType::from(task_type),
        proof_hash: r.try_get_unchecked("proof_hash")?,
        proof_id: r.try_get_unchecked::<Option<i64>, &str>("proof_id")?.map(|id| id as u64),
        task_status: TaskStatus::from(task_status)
    };
    Ok(task)
}

pub async fn update_task_status(pool: &Pool<Any>, task_id: u64, task_status: TaskStatus) -> AnyhowResult<()> {
    let query  = sqlx::query("UPDATE task set task_status = ? where id = ?")
                .bind(task_status.as_u8() as i64).bind(task_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", task_status.as_u8(), task_id);

    let row_affected = match query.execute(pool).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    row_affected
}

pub async fn get_aggregation_waiting_tasks_num(pool: &Pool<Any>) -> Result<u64, Error> {
    let query  = sqlx::query("SELECT Count(*) as reduced_proof_count from task where task_status = ? and task_type = ? ")
                .bind(TaskStatus::Completed.as_u8() as i64).bind(TaskType::ProofGeneration.as_u8() as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}", TaskStatus::Completed.as_u8(), TaskType::ProofGeneration.as_u8());

    let reduction_circuit = match query.fetch_one(pool).await{
        Ok(t) =>{ 
            let id: u64 = t.try_get_unchecked::<i64, &str>("reduced_proof_count")?.try_into()?;
            Ok(id)
        }
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    reduction_circuit
} 

pub async fn create_proof_task(pool: &Pool<Any>, user_circuit_hash: &str, task_type: TaskType, task_status: TaskStatus, proof_hash: &str, proof_id: u64) -> AnyhowResult<u64, Error> {
    let query  = sqlx::query("INSERT into task(user_circuit_hash, task_type, task_status, proof_hash, proof_id) VALUES(?,?,?,?,?)")
                .bind(user_circuit_hash).bind(task_type.as_u8() as i64).bind(task_status.as_u8() as i64).bind(proof_hash).bind(proof_id as i64);

    info!("{}", query.sql());
    info!("arguments: {}, {}, {}, {}, {}", user_circuit_hash, task_type.as_u8(), task_status.as_u8(), proof_hash, proof_id);
    
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => {
            info!("error in db: {:?}", error_line!(e));
            Err(anyhow!(CustomError::DB(error_line!(e))))
        }
    };
    row_affected
}