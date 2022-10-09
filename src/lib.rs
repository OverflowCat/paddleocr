use std::io::Result as IoResult;
use std::io::{BufRead, BufReader, Write};
use std::process;
use std::{error::Error, ffi::OsString, fmt, path::PathBuf};

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
        if !cfg!(target_os = "windows") {
            return Err(Box::new(OsNotSupportedError {}));
        }

        println!("{:?}", std::env::current_dir()?);
        let wd = OsString::from(exe_path.parent().unwrap());
        std::env::set_current_dir(&wd).unwrap();
        let process = process::Command::new(&exe_path)
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

        let mut p = Ppocr { exe_path, process };

        for _i in 1..20 {
            let mut stdout = BufReader::new(p.process.stdout.as_mut().unwrap());
            match p.read_line() {
                Ok(line) => {
                    if line.contains("OCR init completed.")
                        || line.contains("Image path dose not exist")
                    {
                        // initialization successful
                        break;
                    } else if line.contains("PaddleOCR-json v1.2.1") {
                        // in v1.2.1 the last line cannot be read by read_line
                        p.write_fmt(format_args!("\n")).err();
                    }
                }
                Err(e) => {
                    return Err(Box::new(e));
                }
            }
        }

        Ok(p)
    }

    fn read_line(&mut self) -> IoResult<String> {
        let mut buff = String::new();
        let mut stdout = BufReader::new(self.process.stdout.as_mut().unwrap());
        match stdout.read_line(&mut buff) {
            Ok(_siz) => Ok(buff),
            Err(e) => Err(e),
        }
    }

    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> IoResult<()> {
        let stdin = self.process.stdin.as_mut().unwrap();
        stdin.write_fmt(fmt)
    }

    pub fn ocr<S: AsRef<str> + std::fmt::Display>(&mut self, image_path: S) -> IoResult<String> {
        self.write_fmt(format_args!("{}\n", &image_path))?;
        self.read_line()
    }

    pub fn ocr_clipboard(&mut self) -> IoResult<String> {
        self.ocr("clipboard")
    }
}

impl Drop for Ppocr {
    fn drop(&mut self) {
        self.process.kill().err();
    }
}
