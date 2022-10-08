use std::{error::Error, ffi::OsString, fmt, path::PathBuf, time::Duration};

use subprocess::{Popen, PopenConfig, Redirection};

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
            println!("{:?}", std::env::current_dir()?);
            let wd = OsString::from(exe_path.parent().unwrap());
            println!("{:?}", wd);
            std::env::set_current_dir(&wd).unwrap();
            println!("{:?}", std::env::current_dir()?);
            let mut p = Popen::create(
                // &["python3", "hello.py"],
                &[
                    exe_path.to_str().unwrap(),
                    "--det_model_dir=ch_PP-OCRv3_det_infer",
                    "--cls_model_dir=ch_ppocr_mobile_v2.0_cls_infer",
                    "--rec_model_dir=ch_PP-OCRv3_rec_infer",
                    " --rec_char_dict_path=ppocr_keys_v1.txt",
                ],
                PopenConfig {
                    stdin: Redirection::Pipe,
                    stdout: Redirection::Pipe,
                    // stderr: Redirection::Pipe,
                    // executable: Some(OsString::from(format!("{}{}", exe_path.to_str().unwrap(), "")),),
                    detached: true,
                    ..Default::default()
                },
            )?;
            let mut counter = 0;
            loop {
                if let Some(exit_status) = p.poll() {
                    println!("exit_status: {:?}", exit_status);
                    break;
                }
                let mut comm = p
                    .communicate_start(Some(
                        "C:\\Users\\Neko\\Pictures\\boosi.png".as_bytes().to_vec(),
                    ))
                    .limit_time(Duration::from_secs(3));
                let (out, err) = comm.read().unwrap();
                let out = format!("{:?}", &out.unwrap());
                println!("{}", out);
            }

            Ok(Ppocr { exe_path })
        } else {
            Err(Box::new(OsNotSupportedError))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::Ppocr;
    #[test]
    fn it_works() {
        let api = Ppocr::new(PathBuf::from(
            // "C:/Users/Neko/Documents/GitHub/cwdinspect/target/release/cwdinspect.exe"
            "C:/Users/Neko/Documents/GitHub/paddleocr/PaddleOCR-json/PaddleOCR_json.exe",
        ));
        api.unwrap();
    }
}
