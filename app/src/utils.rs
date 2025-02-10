use std::{fs::File, io::{self, Read}};

pub fn read_file_to_vec(filename: &str) -> Result<Vec<u8>, io::Error> {
    // Try to open the file and error out on failure.
    let file_result = File::open(filename);
    let mut file = match file_result {
        Ok(file) => file,
        Err(e) => return Err(e)
    };
    
    // Read the entire contents of the file.
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    Ok(buffer)
}