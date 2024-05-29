use quantum_types::types::db::protocol::Protocol;
use sqlx::{mysql::MySqlRow, Error, Execute, MySql, Pool, Row};
use tracing::info;
use anyhow::Result as AnyhowResult;

pub async fn get_protocol_by_auth_token(pool: &Pool<MySql>, auth_token: &str) -> AnyhowResult<Option<Protocol>> {
     // oldest_entry(task_status: TaskStatus::NotPicked)
     let query  = sqlx::query("SELECT * from protocol where auth_token = ?")
     .bind(auth_token);
    
    info!("{}", query.sql());
    let protocol = match query.fetch_optional(pool).await? {
        Some(r) => Some(get_protocol_from_row(r)?),
        None => None,
    };
    Ok(protocol)
}

pub fn get_protocol_from_row(row: MySqlRow) -> AnyhowResult<Protocol>{
    println!("{:?}", row);
    Ok( 
        Protocol {
            protocol_name: row.try_get_unchecked("protocol_name")?,
            auth_token: row.try_get_unchecked("auth_token")?,
        }
    )
}


pub async fn check_if_protocol_already_registered(pool: &Pool<MySql>, protocol_name: &str) -> AnyhowResult<bool> {
    let query  = sqlx::query("SELECT * from protocol where protocol_name = ?")
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
    let query  = sqlx::query("INSERT into protocol(protocol_name, auth_token) VALUES(?,?)")
        .bind(protocol_name).bind(auth_token);

    info!("{}", query.sql());
    let row_affected = match query.execute(pool).await {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}