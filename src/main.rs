mod png;

use png::PngImage;

fn main() {
    match PngImage::new("./test.png") {
        Ok(image) => println!("data: {}", image),
        Err(e) => println!("error: {:?}", e.get_message()),
    }
}
