use core::panic;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::{error::Error, ffi::OsString, fmt, path::PathBuf, time::Duration};

use std::process;

#[derive(Debug, Clone)]
pub struct OsNotSupportedError;
impl fmt::Display for OsNotSupportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OS not supported")
    }
}
impl Error for OsNotSupportedError {}
pub struct Ppocr {
    exe_path: PathBuf,
    process: process::Child,
}

impl Ppocr {
    pub fn new(exe_path: PathBuf) -> Result<Ppocr, Box<dyn Error>> {
        std::env::set_var("RUST_BACKTRACE", "full");
        if cfg!(target_os = "windows") {
        } else {
            return Err(Box::new(OsNotSupportedError {}));
        }

        println!("{:?}", std::env::current_dir()?);
        let wd = OsString::from(exe_path.parent().unwrap());
        std::env::set_current_dir(&wd).unwrap();
        let mut sp = process::Command::new(&exe_path)
            .args(&[
                "--det_model_dir=ch_PP-OCRv3_det_infer",
                "--cls_model_dir=ch_ppocr_mobile_v2.0_cls_infer",
                "--rec_model_dir=ch_PP-OCRv3_rec_infer",
                " --rec_char_dict_path=ppocr_keys_v1.txt",
            ])
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .stdin(process::Stdio::piped())
            .spawn()?;

        let mut sstdout = BufReader::new(sp.stdout.as_mut().unwrap());
        // let mut sstderr = BufReader::new(sp.stderr.as_mut().unwrap());
        let sstdin = sp.stdin.as_mut().unwrap();

        // initializing
        let mut buff = String::new();
        for _i in 1..8 {
            match sstdout.read_line(&mut buff) {
                Ok(_) => {
                    println!("《{}》", buff);
                    if buff.starts_with("OCR init completed.") {
                        println!("OCR 初始化成功！");
                        let image_path = "C:\\Users\\Neko\\Pictures\\test.png";
                        sstdin.write_fmt(format_args!("{}\n", image_path)).unwrap();
                    }
                    buff.clear();
                }
                Err(e) => {
                    println!("读取 stdout 发生错误：{:?}", e);
                    panic!()
                }
            }
        }
        Ok(Ppocr {
            exe_path,
            process: sp,
        })
    }
    pub fn read_line() {}
    pub fn write() {}
    pub fn ocr(image_path: PathBuf) -> String {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    #[test]
    fn it_works() {
        let api = Ppocr::new(PathBuf::from(
            "C:/Users/Neko/Documents/GitHub/paddleocr/PaddleOCR-json/PaddleOCR_json.exe",
        ));
        api.unwrap();
    }
}
