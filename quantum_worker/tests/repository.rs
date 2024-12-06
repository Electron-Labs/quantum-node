use anyhow::{anyhow, Error};
use quantum_db::{error::error::CustomError, repository::task_repository::get_task_from_mysql_row};
use quantum_types::types::db::task::Task;
use quantum_utils::error_line;
use sqlx::{MySql, Pool};
use tracing::info;

pub async fn get_task_by_task_id(pool: &Pool<MySql>, task_id: u64) -> Result<Task, Error> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    let query  = sqlx::query("SELECT * from task where id = ?")
                .bind(task_id);

    // info!("{}", query.sql());

    let task = match query.fetch_one(pool).await{
        Ok(t) => {
            let row = get_task_from_mysql_row(t)?;
            Ok(row)
        },
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    task
}