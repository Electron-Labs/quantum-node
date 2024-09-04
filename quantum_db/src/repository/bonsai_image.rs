use sqlx::{MySql, Pool, Row, Execute};
use quantum_types::types::db::reduction_circuit::ReductionCircuit;

use anyhow::{anyhow, Result as AnyhowResult, Error as AnyhowError};
use sqlx::mysql::MySqlRow;
use tracing::info;
use quantum_types::enums::circuit_reduction_status::CircuitReductionStatus;
use quantum_types::enums::proving_schemes::ProvingSchemes;
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

fn get_bonsai_image_from_mysql_row(row: &MySqlRow) -> AnyhowResult<BonsaiImage, AnyhowError>{
    let proving_scheme = match ProvingSchemes::from_str(row.try_get_unchecked("proving_scheme").map_err(|err| anyhow!(error_line!(err)))?) {
        Ok(ps) => Ok(ps),
        Err(e) => Err(anyhow!(CustomError::DB(error_line!(e))))
    };
    let bonsai_image = BonsaiImage {
        image_id : row.try_get_unchecked("image_id")?,
        elf_file_path: row.try_get_unchecked("elf_file_path")?,
        circuit_verifying_id: row.try_get_unchecked("circuit_verifying_id")?,
        proving_scheme: proving_scheme?,
        is_aggregation_image_id: row.try_get_unchecked("is_aggregation_image_id")?
    };
    Ok(bonsai_image)
}
