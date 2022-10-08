use std::io::{BufRead, BufReader};
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
        let mut sp = process::Command::new("python3")
            .args(&[
                "-c",
                r#"from time import sleep
sleep(0.01)
print("a", end="_")
sleep(0.2)
print("b")
print('{some: "json!"}')
sleep(1)
print("process ended!!!")"#,
            ])
            .stdout(process::Stdio::piped())
            .stderr(process::Stdio::piped())
            .spawn()?;
        /* exe_path.to_str().unwrap(),
        "--det_model_dir=ch_PP-OCRv3_det_infer",
        "--cls_model_dir=ch_ppocr_mobile_v2.0_cls_infer",
        "--rec_model_dir=ch_PP-OCRv3_rec_infer",
        " --rec_char_dict_path=ppocr_keys_v1.txt", */
        let mut sstdout = BufReader::new(sp.stdout.as_mut().unwrap());
        // let mut sstderr = BufReader::new(sp.stderr.as_mut().unwrap());
        let mut buff = String::new();
        for _i in 1..50 {
            match sstdout.read_line(&mut buff) {
                Ok(siz) => {
                    println!("Read size {}: {}", siz, buff);
                    buff.clear();
                }
                Err(e) => {
                    println!("读取 stdout 发生错误：{:?}", e);
                    break;
                }
            }
        }
        Ok(Ppocr { exe_path })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    #[test]
    fn it_works() {
        let api = Ppocr::new(PathBuf::from(
            // "C:/Users/Neko/Documents/GitHub/cwdinspect/target/release/cwdinspect.exe"
            "C:/Users/Neko/Documents/GitHub/paddleocr/PaddleOCR-json/PaddleOCR_json.exe",
        ));
        api.unwrap();
    }
}
