use std::fs;

pub struct PngImage {
    pub width: u32,
    pub height: u32,
    pub chunk_size: u32,
    pub chunk_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum PngError {
    InvalidHeader,
    InvalidChunk,
    InvalidChunkType,
}

impl PngImage {
    pub fn new(path: &str) -> Result<Self, PngError> {
        let data = read_image_data(path);

        let file_type_bytes = &data[0..8];

        // first 8 bytes of a PNG file specify that it is indeed a PNG file
        if file_type_bytes != [137, 80, 78, 71, 13, 10, 26, 10] {
            return Err(PngError::InvalidHeader);
        }

        let temp_size = &data[8..12];
        let chunk_size = u32::from_be_bytes(array_from_slice(temp_size));

        let chunk_type = match String::from_utf8(data[12..16].to_vec()) {
            Ok(s) => s,
            Err(e) => return Err(PngError::InvalidChunkType),
        };

        format!("byte array: {:?}", data);

        Ok(PngImage {
            width: 0,
            height: 0,
            chunk_size: chunk_size,
            chunk_type: chunk_type,
            data: data,
        })
    }
}

fn array_from_slice(slice: &[u8]) -> [u8; 4] {
    slice.try_into().unwrap()
}

fn read_image_data(file_path: &str) -> Vec<u8> {
    match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => vec![],
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
