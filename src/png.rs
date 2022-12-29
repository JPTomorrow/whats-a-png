use std::{
    fmt::{Display, Formatter},
    fs,
    io::{Cursor, Read, Seek, SeekFrom},
};

pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub chunk_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum PngError {
    InvalidFileType,
    InvalidChunk,
    CannotSkipChunk,
    InvalidChunkType,
    InvalidChunkSize,
}

impl PngError {
    pub fn get_message(&self) -> String {
        match self {
            PngError::InvalidFileType => "Invalid file type".to_string(),
            PngError::InvalidChunk => "Invalid chunk".to_string(),
            PngError::CannotSkipChunk => "Cannot skip chunk".to_string(),
            PngError::InvalidChunkType => "Invalid chunk type".to_string(),
            PngError::InvalidChunkSize => "Invalid chunk size".to_string(),
        }
    }
}

impl PngImage {
    pub fn new(path: &str) -> Result<Self, PngError> {
        let mut data = Cursor::new(read_image_data(path));
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

        // bytes 8-12 specify the size of the chunk
        let mut chunk_size = [0; 4];
        let res = data.read(&mut chunk_size);

        if res.is_err() {
            return Err(PngError::InvalidChunkSize);
        }

        let chunk_size = u32::from_be_bytes(chunk_size);

        // IHDR for head chunk and IEND for end chunk
        let mut chunk_type_buf = [0; 4];
        let res = data.read(&mut chunk_type_buf);

        if res.is_err() {
            return Err(PngError::InvalidChunkType);
        }

        let chunk_type = match String::from_utf8(chunk_type_buf.to_vec()) {
            Ok(s) => s,
            Err(e) => return Err(PngError::InvalidChunkType),
        };

        Ok(PngImage {
            width: 0,
            height: 0,
            chunk_size: chunk_size,
            chunk_type: chunk_type,
            data: data.into_inner(),
        })
    }
}

impl Display for PngImage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "width: {}, height: {}, chunk_size: {}, chunk_type: {}",
            self.width, self.height, self.chunk_size, self.chunk_type
        )
    }
}

fn read_image_data(file_path: &str) -> Vec<u8> {
    match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(_) => vec![],
    }
}

// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_image_data() {
        let data = read_image_data("./test.png");
        assert_eq!(data.len(), 226933);
    }

    #[test]
    fn test_new() {
        PngImage::new("./test.png").unwrap();
    }

    #[test]
    fn test_chunk_size() {
        let image = PngImage::new("./test.png").unwrap();
        assert_eq!(image.chunk_size, 13);
    }

    #[test]
    fn test_chunk_type() {
        let image = PngImage::new("./test.png").unwrap();
        assert_eq!(image.chunk_type, "IHDR");
    }
}
