use std::{
    fmt::{Display, Formatter},
    fs::{self, File},
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

#[derive(Debug)]
pub enum PngError {
    InvalidFileType,
    InvalidChunk,
    CannotSkipChunk,
    InvalidChunkType(String),
    InvalidChunkCrc(String), // cyclic redundancy check
    SaveOperationFailed,
    InvalidChunkSize,
}

impl PngError {
    pub fn get_message(&self) -> String {
        match self {
            PngError::InvalidFileType => "Invalid file type".to_string(),
            PngError::InvalidChunk => "Invalid chunk".to_string(),
            PngError::CannotSkipChunk => "Cannot skip chunk".to_string(),
            PngError::InvalidChunkType(s) => format!("Invalid chunk type: {}", s),
            PngError::InvalidChunkCrc(s) => format!("Invalid chunk crc: {}", s),
            PngError::SaveOperationFailed => "Save operation failed".to_string(),
            PngError::InvalidChunkSize => "Invalid chunk size".to_string(),
        }
    }
}

pub struct PngImage {
    pub chunks: Vec<PNGChunk>,
}

#[derive(Debug)]
pub struct PNGChunk {
    pub size: u32,
    pub chunk_type: String,
    pub data: Vec<u8>,
    pub crc: u32,
}

impl PngImage {
    pub fn new(path: &str) -> Result<Self, PngError> {
        let mut data = Cursor::new(read_image_data(path));

        // check file type
        if Self::check_file_type(&mut data).is_err() {
            return Err(PngError::InvalidFileType);
        }

        let mut chunks = vec![];
        let mut is_proccessing_chunk = true;

        while is_proccessing_chunk {
            let chunk_size = match Self::get_chunk_size(&mut data) {
                Ok(size) => size,
                Err(e) => return Err(e),
            };

            let chunk_type = match Self::get_chunk_type(&mut data) {
                Ok(s) => s,
                Err(e) => return Err(e),
            };

            let chunk_data = match Self::get_chunk_data(&mut data, chunk_size) {
                Ok(data) => data,
                Err(e) => return Err(e),
            };

            let chunk_crc = match Self::get_chunk_crc(&mut data) {
                Ok(crc) => crc,
                Err(e) => return Err(e),
            };

            let chunk = PNGChunk {
                size: chunk_size,
                chunk_type: chunk_type,
                data: chunk_data,
                crc: chunk_crc,
            };

            if chunk.chunk_type.eq("IEND") {
                is_proccessing_chunk = false;
            }

            chunks.push(chunk);
        }

        Ok(PngImage { chunks })
    }

    fn get_chunk_data(data: &mut Cursor<Vec<u8>>, size: u32) -> Result<Vec<u8>, PngError> {
        let mut chunk_data = vec![0; size as usize];
        let res = data.read(&mut chunk_data);

        if res.is_err() {
            return Err(PngError::InvalidChunk);
        }

        Ok(chunk_data)
    }

    fn get_chunk_crc(data: &mut Cursor<Vec<u8>>) -> Result<u32, PngError> {
        let mut chunk_type_buf = [0; 4];
        let res = data.read(&mut chunk_type_buf);

        if res.is_err() {
            return Err(PngError::InvalidChunkCrc(
                "Failed to read 4 byte CRC hash".to_string(),
            ));
        }

        Ok(u32::from_be_bytes(chunk_type_buf))
    }

    fn get_chunk_type(data: &mut Cursor<Vec<u8>>) -> Result<String, PngError> {
        // IHDR for head chunk and IEND for end chunk
        let mut chunk_type_buf = [0; 4];
        let res = data.read(&mut chunk_type_buf);

        if res.is_err() {
            return Err(PngError::InvalidChunkType(
                "Failed to read 4 bytes out of data buffer".to_string(),
            ));
        }

        match String::from_utf8(chunk_type_buf.to_vec()) {
            Ok(s) => Ok(s),
            Err(_) => Err(PngError::InvalidChunkType(
                "Failed to convert chunk type to string".to_string(),
            )),
        }
    }

    fn get_chunk_size(data: &mut Cursor<Vec<u8>>) -> Result<u32, PngError> {
        // bytes 8-12 specify the size of the chunk
        let mut chunk_size = [0; 4];
        let res = data.read(&mut chunk_size);

        if res.is_err() {
            return Err(PngError::InvalidChunkSize);
        }

        Ok(u32::from_be_bytes(chunk_size))
    }

    fn check_file_type(data: &mut Cursor<Vec<u8>>) -> Result<(), PngError> {
        let res = data.seek(SeekFrom::Start(0));

        if res.is_err() {
            return Err(PngError::InvalidFileType);
        }

        // check file type
        let mut file_type_bytes = [0; 8];
        let res = data.read(&mut file_type_bytes);

        if res.is_err() {
            return Err(PngError::InvalidFileType);
        }

        // first 8 bytes of a PNG file specify that it is indeed a PNG file
        if file_type_bytes != [137, 80, 78, 71, 13, 10, 26, 10] {
            return Err(PngError::InvalidFileType);
        }

        Ok(())
    }

    pub fn save_image(self: &Self, path: &str) -> Result<(), PngError> {
        let mut file = match File::create(path) {
            Ok(f) => f,
            Err(_) => return Err(PngError::SaveOperationFailed),
        };

        let mut bytes: Vec<u8> = vec![137, 80, 78, 71, 13, 10, 26, 10];
        for chunk in &self.chunks {
            bytes.extend_from_slice(&chunk.size.to_be_bytes());
            bytes.extend_from_slice(&chunk.chunk_type.as_bytes());
            bytes.extend_from_slice(&chunk.data);
            bytes.extend_from_slice(&chunk.crc.to_be_bytes());
        }

        match file.write(&bytes) {
            Ok(_) => (),
            Err(_) => return Err(PngError::SaveOperationFailed),
        };

        Ok(())
    }
}

impl Display for PngImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.to_string())
    }
}

fn read_image_data(file_path: &str) -> Vec<u8> {
    match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(_) => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const IMAGE_PATH: &str = "./test.png";

    #[test]
    fn test_read_image_data() {
        let data = read_image_data(IMAGE_PATH);
        assert_eq!(data.len(), 226933);
    }

    #[test]
    fn test_new() {
        PngImage::new(IMAGE_PATH).unwrap();
    }

    #[test]
    fn test_chunks() {
        let image = PngImage::new(IMAGE_PATH).unwrap();

        // check sizes
        assert_eq!(image.chunks[0].size, 13);
        assert_eq!(image.chunks[1].size, 226876);
        assert_eq!(image.chunks[2].size, 0);

        // check chunk types
        assert_eq!(image.chunks[0].chunk_type, "IHDR");
        assert_eq!(image.chunks[1].chunk_type, "IDAT");
        assert_eq!(image.chunks[2].chunk_type, "IEND");

        // check chunk crcs
        assert_eq!(image.chunks[0].crc, 0x9A768270);
        assert_eq!(image.chunks[1].crc, 0x177D762A);
        assert_eq!(image.chunks[2].crc, 0xAE426082);
        assert_eq!(image.chunks.len(), 3);
    }

    #[test]
    fn test_save_image() {
        let image = PngImage::new(IMAGE_PATH).unwrap();
        image.save_image("./save_test/test_copy.png").unwrap();
    }
}
