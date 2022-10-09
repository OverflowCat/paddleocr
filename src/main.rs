use std::path::PathBuf;

pub mod lib;
fn main() {
    let mut p = lib::Ppocr::new(std::path::PathBuf::from(
        "E:\\code\\paddleocr\\PaddleOCR-json\\PaddleOCR_json.exe",
    ))
    .unwrap();
    let image = PathBuf::from("C:\\Users\\Neko\\Pictures\\test.png");
    println!("{}", p.ocr(&image).unwrap());
}
