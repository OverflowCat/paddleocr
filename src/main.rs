pub mod lib;
fn main() {
    let p = lib::Ppocr::new(std::path::PathBuf::from(
        "E:\\code\\paddleocr\\PaddleOCR-json\\PaddleOCR_json.exe",
    )).unwrap();
}
