mod png;

use png::PngImage;

fn main() {
    match PngImage::new("./test.png") {
        Ok(image) => println!("data: {:?}", image.data),
        Err(e) => panic!("Error: {:?}", e),
    }
}
