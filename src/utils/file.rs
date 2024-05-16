use std::{fs::{self, File}, io::{BufWriter, Write}};

use serde::Serialize;

pub fn dump_json_file<T: Serialize>(file_path: &str, file_name: &str, value: T){
    let file = File::create(
        format!("{}/{}",file_path, file_name ).as_str(),
    )
    .unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &value).unwrap();
    writer.flush().unwrap();
}

pub fn create_dir(full_path: &str) {
    fs::create_dir_all(full_path).unwrap();
}