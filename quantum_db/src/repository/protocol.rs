use quantum_types::types::db::protocol::Protocol;
use sqlx::{mysql::MySqlRow, Error, Execute, MySql, Pool, Row};
use tracing::info;
use anyhow::{Context, Result as AnyhowResult};

pub async fn get_protocol_by_auth_token(pool: &Pool<MySql>, auth_token: &str) -> AnyhowResult<Option<Protocol>> {
     // oldest_entry(task_status: TaskStatus::NotPicked)
     let query  = sqlx::query("SELECT * from protocol where auth_token = ?")
     .bind(auth_token);
    
    info!("{}", query.sql());
    let protocol = match query.fetch_optional(pool).await.with_context(|| format!("Error: unable to fetch query in file: {} on line: {}", file!(), line!()))? {
        Some(r) => Some(get_protocol_from_row(r).with_context(|| format!("Error: cannot get protocol from row in file: {} on line: {}", file!(), line!()))?),
        None => None,
    };
    Ok(protocol)
}

pub fn get_protocol_from_row(row: MySqlRow) -> AnyhowResult<Protocol>{
    println!("{:?}", row);
    Ok( 
        Protocol {
            protocol_name: row.try_get_unchecked("protocol_name").with_context(|| format!("Error: no column named protocol in mysql row in file: {} on line: {}", file!(), line!()))?,
            auth_token: row.try_get_unchecked("auth_token").with_context(|| format!("Error: no column named auth_token in mysql row in file: {} on line: {}", file!(), line!()))?,
        }
    )
}


pub async fn check_if_protocol_already_registered(pool: &Pool<MySql>, protocol_name: &str) -> AnyhowResult<bool> {
    let query  = sqlx::query("SELECT * from protocol where protocol_name = ?")
        .bind(protocol_name);

   info!("{}", query.sql());
   let is_present = match query.fetch_optional(pool).await.with_context(|| format!("Error: cannot verify if protocol is registered in file: {} on line: {}", file!(), line!()))?{
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
    let row_affected = match query.execute(pool).await.with_context(|| format!("Error: cannot get affected mysql rows in file: {} on line: {}", file!(), line!())) {
        Ok(t) => Ok(t.rows_affected()),
        Err(e) => Err(e)
    };
    row_affected
}