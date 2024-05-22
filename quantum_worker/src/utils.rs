use std::fs::File;
use std::io::{Read, Write};
use anyhow::Result as AnyhowResult;

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

#[cfg(test)]
mod tests {
    use super::{read_bytes_from_file, write_bytes_to_file};


    #[test]
    pub fn test_read_write() {
        let bytes_vec: Vec<u8> = vec![0x48, 0x65, 0x6c, 0x6c, 0x6f];
        write_bytes_to_file(&bytes_vec, "./test.bytes").expect("Failed to write bytes to file");
        let read_bytes_vec = read_bytes_from_file("./test.bytes").unwrap();
        assert_eq!(read_bytes_vec, bytes_vec);
    }
}