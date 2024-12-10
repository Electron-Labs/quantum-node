use sqlx::{mysql::{MySqlConnectOptions, MySqlPoolOptions, MySqlQueryResult, MySqlRow}, ConnectOptions, Executor, MySql, Pool};
use anyhow::Result as AnyhowResult;
use crate::traits::db::{DatabaseQueryResult, DatabaseRow, DB};


pub struct MySqlDB {
    pub pool: Pool<MySql>
}
// impl DB {
//     fn form_query( query: &str, arguments: Vec<&str>) -> Query<'_, MySql, MySqlArgument> {
//         let mut query = sqlx::query(query);
//         for arg in arguments {
//             query = query.bind(arg);
//         }
//         query
//     }
// }
impl DB for MySqlDB {
    // async fn fetch_optional(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Option<MySqlRow>> {
    //     let mut query = sqlx::query(query);
    //     for arg in arguments {
    //         query = query.bind(arg);
    //     }
    //     let result = match self.pool.fetch_optional(query).await? {
    //         Some(row) => Some(row),
    //         None => None,
    //     };
    //     Ok(result)
    // }

    // async fn fetch_one(&self, query: &str, arguments: Vec<&str>) -> Result<MySqlRow, sqlx::Error> {
    //     let mut query = sqlx::query(query);
    //     for arg in arguments {
    //         query = query.bind(arg);
    //     }
    //     query.fetch_one(&self.pool).await
    //     // match self.pool.fetch_one(query).await {
    //     //     Ok(r) => Ok(r),
    //     //     Err(e) => Err(e.into()),
    //     // }
    // }

    // async fn execute(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<MySqlQueryResult> {
    //     let mut query = sqlx::query(query);
    //     for arg in arguments {
    //         query = query.bind(arg);
    //     }
    //     match self.pool.execute(query).await {
    //         Ok(row) => Ok(row),
    //         Err(e) => Err(e.into()),
    //     }
    // }

    // async fn fetch_all(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Vec<MySqlRow>> {
    //     let mut query = sqlx::query(query);
    //     for arg in arguments {
    //         query = query.bind(arg);
    //     }
    //     let rows_vec = self.pool.fetch_all(query).await?;
    //     // let mut db_row = vec![];
    //     // for row in rows_vec {
    //     //     db_row.push(DatabaseRow::Mysql(row));
    //     // }
    //     Ok(rows_vec)
    // }

    // async fn initialize_pool() -> Self {
    //     let username = std::env::var("DB_USER").expect("DB_USER must be set.");
    //     let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
    //     let database = std::env::var("DB_NAME").expect("DB_NAME must be set.");

    //     let connection_options = MySqlConnectOptions::new()
    //         .username(&username)
    //         .password(&password)
    //         .database(&database)
    //         .disable_statement_logging().clone();

    //     let pool_options = MySqlPoolOptions::new().min_connections(5);
    //    let pool = pool_options.connect_with(connection_options).await.unwrap();
    //    MySqlDB {pool}
    // }
}










// pub struct SqliteDB {
//     pub pool: Pool<MySql>
// }

// impl DB for SqliteDB {
//     async fn fetch_optional(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Option<DatabaseRow>> {
//         let mut query = sqlx::query(query);
//         for arg in arguments {
//             query = query.bind(arg);
//         }
//         let result = match self.pool.fetch_optional(query).await? {
//             Some(row) => Some(DatabaseRow::Mysql(row)),
//             None => None,
//         };
//         Ok(result)
//     }

//     async fn fetch_one(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<DatabaseRow> {
//         let mut query = sqlx::query(query);
//         for arg in arguments {
//             query = query.bind(arg);
//         }
//         match self.pool.fetch_one(query).await {
//             Ok(r) => Ok(DatabaseRow::Mysql(r)),
//             Err(e) => Err(e.into()),
//         }
//     }

//     async fn execute(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<DatabaseQueryResult> {
//         let mut query = sqlx::query(query);
//         for arg in arguments {
//             query = query.bind(arg);
//         }
//         match self.pool.execute(query).await {
//             Ok(row) => Ok(DatabaseQueryResult::Mysql(row)),
//             Err(e) => Err(e.into()),
//         }
//     }

//     async fn fetch_all(&self, query: &str, arguments: Vec<&str>) -> AnyhowResult<Vec<DatabaseRow>> {
//         let mut query = sqlx::query(query);
//         for arg in arguments {
//             query = query.bind(arg);
//         }
//         let rows_vec = self.pool.fetch_all(query).await?;
//         let mut db_row = vec![];
//         for row in rows_vec {
//             db_row.push(DatabaseRow::Mysql(row));
//         }
//         Ok(db_row)
//     }

//     // async fn initialize_pool() -> Self {
//     //     let username = std::env::var("DB_USER").expect("DB_USER must be set.");
//     //     let password = std::env::var("DB_PASSWORD").expect("DB_PASSWORD must be set.");
//     //     let database = std::env::var("DB_NAME").expect("DB_NAME must be set.");

//     //     let connection_options = MySqlConnectOptions::new()
//     //         .username(&username)
//     //         .password(&password)
//     //         .database(&database)
//     //         .disable_statement_logging().clone();

//     //     let pool_options = MySqlPoolOptions::new().min_connections(5);
//     //    let pool = pool_options.connect_with(connection_options).await.unwrap();
//     //    SqliteDB {pool}
//     // }
// }