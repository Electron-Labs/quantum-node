use std::{fs::{self, File}, io::{BufWriter, Write}};

use serde::Serialize;

use anyhow::Result as AnyhowResult;

pub fn dump_json_file<T: Serialize>(file_path: &str, file_name: &str, value: T) -> AnyhowResult<()>{
    let file = File::create(
        format!("{}/{}",file_path, file_name ).as_str(),
    )?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &value)?;
    writer.flush()?;
    Ok(())
}

pub fn create_dir(full_path: &str) -> AnyhowResult<()>{
    let res = fs::create_dir_all(full_path)?;
    Ok(res)
}