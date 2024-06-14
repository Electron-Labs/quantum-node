use quantum_types::{types::db::protocol::Protocol};
use quantum_utils::error_line;
use sqlx::{mysql::MySqlRow, Execute, MySql, Pool, Row};
use tracing::info;
use anyhow::{anyhow, Result as AnyhowResult};

pub async fn get_protocol_by_auth_token(pool: &Pool<MySql>, auth_token: &str) -> AnyhowResult<Option<Protocol>> {
     // oldest_entry(task_status: TaskStatus::NotPicked)
     let query  = sqlx::query("SELECT * from protocol where auth_token = ?")
     .bind(auth_token);
    
    info!("{}", query.sql());
    let protocol = match query.fetch_optional(pool).await.map_err(|err| anyhow!(error_line!(err)))? {
        Some(r) => Some(get_protocol_from_row(r)?),
        None => None,
    };
    Ok(protocol)
}

pub fn get_protocol_from_row(row: MySqlRow) -> AnyhowResult<Protocol>{
    println!("{:?}", row);
    Ok( 
        Protocol {
            protocol_name: row.try_get_unchecked("protocol_name").map_err(|err| anyhow!(error_line!(err)))?,
            auth_token: row.try_get_unchecked("auth_token").map_err(|err| anyhow!(error_line!(err)))?,
        }
    )
}


pub async fn check_if_protocol_already_registered(pool: &Pool<MySql>, protocol_name: &str) -> AnyhowResult<bool> {
    let query  = sqlx::query("SELECT * from protocol where protocol_name = ?")
        .bind(protocol_name);

   info!("{}", query.sql());
   let is_present = match query.fetch_optional(pool).await.map_err(|err| anyhow!(error_line!(err)))?{
       Some(t) => {
           println!("{:?}", t);
           true
       }
       None => false,
   };
   Ok(is_present)
}

pub async fn insert_protocol_auth_token(pool: &Pool<MySql>, protocol_name: &str, auth_token: &str) -> AnyhowResult<u64, anyhow::Error>{
    let query  = sqlx::query("INSERT into protocol(protocol_name, auth_token) VALUES(?,?)")
        .bind(protocol_name).bind(auth_token);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(anyhow!(error_line!(e)))
    };
    row_affected
}