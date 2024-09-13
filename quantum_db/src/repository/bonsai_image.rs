use sqlx::{MySql, Pool, Row, Execute};
use quantum_types::types::db::reduction_circuit::ReductionCircuit;

use anyhow::{anyhow, Result as AnyhowResult, Error as AnyhowError};
use sqlx::mysql::MySqlRow;
use tracing::info;
use quantum_types::enums::circuit_reduction_status::CircuitReductionStatus;
use quantum_types::enums::proving_schemes::{self, ProvingSchemes};
use quantum_types::types::db::bonsai_image::BonsaiImage;
use quantum_types::types::db::user_circuit_data::UserCircuitData;
use quantum_utils::error_line;
use crate::error::error::CustomError;
use std::str::FromStr;

pub async fn get_bonsai_image_by_proving_scheme(pool: &Pool<MySql>, proving_scheme: ProvingSchemes) -> AnyhowResult<BonsaiImage>{
    let query  = sqlx::query("SELECT * from bonsai_image where proving_scheme = ?")
        .bind(proving_scheme.to_string());

    info!("{}", query.sql());
    info!("arguments: {:?}", proving_scheme);

    let bonsai_image = match query.fetch_one(pool).await{
        Ok(t) => get_bonsai_image_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    bonsai_image
}

pub async fn get_bonsai_image_by_image_id(pool: &Pool<MySql>, image_id: &str) -> AnyhowResult<BonsaiImage>{
    let query  = sqlx::query("SELECT * from bonsai_image where image_id = ?")
        .bind(image_id.to_string());

    info!("{}", query.sql());
    info!("arguments: {:?}", image_id);

    let bonsai_image = match query.fetch_one(pool).await{
        Ok(t) => get_bonsai_image_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    bonsai_image
}

pub async fn get_aggregate_circuit_bonsai_image(pool: &Pool<MySql>) -> AnyhowResult<BonsaiImage> {
    let query  = sqlx::query("SELECT * from bonsai_image where is_aggregation_image_id = 1");

    info!("{}", query.sql());

    let bonsai_image = match query.fetch_one(pool).await{
        Ok(t) => get_bonsai_image_from_mysql_row(&t),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    bonsai_image
}

fn get_bonsai_image_from_mysql_row(row: &MySqlRow) -> AnyhowResult<BonsaiImage, AnyhowError>{
    let proving_scheme_string: Option<String> = row.try_get_unchecked("proving_scheme")?;
    let mut proving_scheme: Option<ProvingSchemes> = None;
    if proving_scheme_string.is_some() {
        proving_scheme = Some(match ProvingSchemes::from_str(&proving_scheme_string.unwrap()) {
            Ok(ps) => Ok(ps),
            Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
        }?);
    }
    
    let circuit_verifying_id_string: String = row.try_get_unchecked("circuit_verifying_id")?;
    let circuit_verifying_id = parse_string_to_u32_array(&circuit_verifying_id_string)?;
    let bonsai_image = BonsaiImage {
        image_id : row.try_get_unchecked("image_id")?,
        elf_file_path: row.try_get_unchecked("elf_file_path")?,
        circuit_verifying_id,
        proving_scheme: proving_scheme,
        is_aggregation_image_id: row.try_get_unchecked("is_aggregation_image_id")?
    };
    Ok(bonsai_image)
}


fn parse_string_to_u32_array(s: &str) -> AnyhowResult<[u32; 8]> {
    // Remove the square brackets and split by commas
    let trimmed = s.trim_matches(|c| c == '[' || c == ']');
    let parsed_values: Result<Vec<u32>, _> = trimmed
        .split(',')
        .map(|num| num.trim().parse::<u32>()) // Trim and parse each number
        .collect();

    match parsed_values {
        Ok(vec) if vec.len() == 8 => {
            let arr: [u32; 8] = vec.try_into().map_err(|_| anyhow!("error"))?;
            Ok(arr)
        },
        Ok(_) => Err(anyhow!(CustomError::DB(error_line!("not able to pares array of u32")))),
        Err(_) => Err(anyhow!(CustomError::DB(error_line!("not able to pares array of u32")))),
    }
}