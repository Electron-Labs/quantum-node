use sqlx::{Error, Execute, MySql, Pool};
use tracing::info;
use anyhow::Result as AnyhowResult;

pub async fn check_if_auth_token_registered(pool: &Pool<MySql>, auth_token: &str) -> AnyhowResult<bool> {
     // oldest_entry(task_status: TaskStatus::NotPicked)
     let query  = sqlx::query("SELECT * from auth where auth_token = ?")
     .bind(auth_token);

    info!("{}", query.sql());
    let is_present = match query.fetch_optional(pool).await?{
        Some(t) => {
            println!("{:?}", t);
            true
        }
        None => false,
    };
    Ok(is_present)
}

pub async fn check_if_auth_token_registered_and_is_master(pool: &Pool<MySql>, auth_token: &str) -> AnyhowResult<bool> {
    // oldest_entry(task_status: TaskStatus::NotPicked)
    let query  = sqlx::query("SELECT * from auth where auth_token = ? and is_master = 1")
    .bind(auth_token);

   info!("{}", query.sql());
   let is_present = match query.fetch_optional(pool).await?{
       Some(t) => {
           println!("{:?}", t);
           true
       }
       None => false,
   };
   Ok(is_present)
}

pub async fn check_if_protocol_already_registered(pool: &Pool<MySql>, protocol_name: &str) -> AnyhowResult<bool> {
    let query  = sqlx::query("SELECT * from auth where protocol_name = ?")
        .bind(protocol_name);

   info!("{}", query.sql());
   let is_present = match query.fetch_optional(pool).await?{
       Some(t) => {
           println!("{:?}", t);
           true
       }
       None => false,
   };
   Ok(is_present)
}

pub async fn insert_protocol_auth_token(pool: &Pool<MySql>, protocol_name: &str, auth_token: &str) -> AnyhowResult<u64, Error>{
    let query  = sqlx::query("INSERT into auth(protocol_name, auth_token) VALUES(?,?)")
        .bind(protocol_name).bind(auth_token);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}