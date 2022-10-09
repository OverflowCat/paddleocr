pub mod lib;
fn main() {
    let mut p = lib::Ppocr::new(std::path::PathBuf::from(
        "E:/code/paddleocr/pojnew/PaddleOCR_json.exe", // path to binary
    ))
    .unwrap(); // initialize

    let now = std::time::Instant::now(); // benchmark
    {
        // OCR files
        println!("{}", p.ocr("C:/Users/Neko/Pictures/test1.png").unwrap());
        println!("{}", p.ocr("C:/Users/Neko/Pictures/test2.png").unwrap());
        println!("{}", p.ocr("C:/Users/Neko/Pictures/test3.png").unwrap());
        println!("{}", p.ocr("C:/Users/Neko/Pictures/test4.png").unwrap());
        println!("{}", p.ocr("C:/Users/Neko/Pictures/test5.png").unwrap());

        // OCR clipboard
        println!("{}", p.ocr_clipboard().unwrap());
    }
    println!("Elapsed: {:.2?}", now.elapsed());
}
