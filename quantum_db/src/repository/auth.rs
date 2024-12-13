use sqlx::{Any, Execute, Pool};
use tracing::info;
use anyhow::Result as AnyhowResult;

pub async fn check_if_auth_token_registered_and_is_master(pool: &Pool<Any>, auth_token: &str) -> AnyhowResult<bool> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    let query  = sqlx::query("SELECT * from auth where auth_token = ? and is_master = 1")
    .bind(auth_token);

   info!("{}", query.sql());
   info!("arguments: {}", auth_token);
   let is_present = match query.fetch_optional(pool).await?{
       Some(_t) => {
           true
       }
       None => false,
   };
   Ok(is_present)
}