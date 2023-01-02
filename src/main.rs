mod png;

use png::PngImage;

fn main() {
    let img = match PngImage::new("./test.png") {
        Ok(image) => image,
        Err(e) => panic!("error: {:?}", e.get_message()),
    };

    img.save_image("./save_test/test_copy-main.png").unwrap();
}
