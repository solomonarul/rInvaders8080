use std::{fs::File, io::Read};

pub fn read_file_to_vec(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer).unwrap();
    buffer
}