# Crate `paddleocr`

A simple wrapper for [`hiroi-sora/PaddleOCR-json`](https://github.com/hiroi-sora/PaddleOCR-json).

## Usage

```rust
let mut p = paddleocr::Ppocr::new(std::path::PathBuf::from(
    ".../PaddleOCR_json.exe", // path to binary
)).unwrap(); // initialize

let now = std::time::Instant::now(); // benchmark
{
    // OCR files
    println!("{}", p.ocr(".../test1.png").unwrap());
    println!("{}", p.ocr(".../test2.png").unwrap());
    println!("{}", p.ocr(".../test3.png").unwrap());

    // OCR clipboard
    println!("{}", p.ocr_clipboard().unwrap());
}
println!("Elapsed: {:.2?}", now.elapsed());
```
