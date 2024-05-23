use std::{fs::{self, File}, io::{BufWriter, Read, Write}};

use keccak_hash::keccak;
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

pub fn dump_object<T: Serialize>(object: T, path: &str, file_name: &str) -> AnyhowResult<()> {
    create_dir(path)?;
    dump_json_file(path, file_name, object)?;
    Ok(())
}

pub fn read_file(path: &str) -> AnyhowResult<String> {
    let data_string = fs::read_to_string(path)?;
    Ok(data_string)
}

// Write bytes to file
pub fn write_bytes_to_file(bytes: &Vec<u8>, path: &str) -> AnyhowResult<()> {
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}

// Read bytes from file
pub fn read_bytes_from_file(path: &str) -> AnyhowResult<Vec<u8>> {
    let mut buffer = Vec::<u8>::new();
    let mut file = File::open(path)?;
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn get_keccak_256_hash(bytes: &Vec<u8>) -> String {
    let mut keccak_ip = bytes.as_slice();
    let hash = keccak(&mut keccak_ip);
    let hash_string = format!("{:?}", hash);
    hash_string
}