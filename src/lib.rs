use std::io::Result as IoResult;
use std::io::{BufRead, BufReader, Write};
use std::process;
use std::{error::Error, fmt, path::PathBuf};

#[derive(Debug, Clone)]
pub struct OsNotSupportedError;
impl fmt::Display for OsNotSupportedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OS not supported")
    }
}
impl Error for OsNotSupportedError {}

#[cfg(feature = "parse")]
use serde::Deserialize;

#[cfg(feature = "parse")]
type Point = [usize; 2];

#[cfg(feature = "parse")]
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OcrRec {
    Content { code: u32, data: Vec<ContentData> },
    Message { code: u32, data: String },
}

#[cfg(feature = "parse")]
#[derive(Deserialize, Debug, Clone)]
pub struct ContentData {
    #[serde(rename(deserialize = "box"))]
    pub rect: Rectangle,
    pub score: f64,
    pub text: String,
}

#[cfg(feature = "parse")]
pub type Rectangle = [Point; 4];
// pub struct Rectangle {
//     topleft: Point,
//     topright: Point,
//     bottomright: Point,
//     bottomleft: Point,
// }

pub struct Ppocr {
    #[allow(dead_code)]
    exe_path: PathBuf,
    process: process::Child,
}

impl Ppocr {
    /**
        Initialize a new instance.

        # Examples

        ```no_run
        let mut p = paddleocr::Ppocr::new(std::path::PathBuf::from(".../PaddleOCR_json.exe",));
        ```
    */
    pub fn new(exe_path: PathBuf) -> Result<Ppocr, Box<dyn Error>> {
        std::env::set_var("RUST_BACKTRACE", "full");
        if !cfg!(target_os = "windows") {
            return Err(Box::new(OsNotSupportedError {}));
        }

        let wd = exe_path
            .canonicalize()?
            .parent()
            .ok_or_else(|| "No parent directory found")?
            .to_path_buf();
        let process = process::Command::new(&exe_path)
            .current_dir(wd)
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

        for _i in 1..10 {
            match p.read_line() {
                Ok(line) => {
                    if line.contains("OCR init completed.")
                        || line.contains("Image path dose not exist")
                    {
                        break; // successfully initialized
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

    /**
    OCRs the image at the given path. Note that the returned JSON is not parsed or checked, and a valid JSON does not necessarily mean it is successful.

    # Examples

    ```no_run
    let mut p = paddleocr::Ppocr::new(std::path::PathBuf::from(
        ".../PaddleOCR_json.exe",
    )).unwrap();
    println!("{}", p.ocr(".../test1.png").unwrap());
    println!("{}", p.ocr(".../test2.png").unwrap());
    println!("{}", p.ocr(".../test3.png").unwrap());
    ```

    # Results

    ## Return values

    根含两个元素：状态码 `code` 和内容 `data` 。

    状态码 `code` 为整数，每种状态码对应一种情况。

    ## Code values

    ### `100` 识别到文字

    - data 为数组。数组每一项为字典，含三个元素：
    - `text`：文本内容，字符串。
    - `box`：文本包围盒，长度为4的数组，分别为左上角、右上角、右下角、左下角的`[x,y]`。整数。
    - `score`：识别置信度，浮点数。
    - 例：
    ```no_run
        {'code':100,'data':[{'box':[[13,5],[161,5],[161,27],[13,27]],'score':0.9996442794799805,'text':'飞舞的因果交流'}]}
    ```

    ### `101` 未识别到文字

    - data 为字符串：`No text found in image. Path:"图片路径"`
    - 例：`{'code':101,'data':'No text found in image. Path: "D:\\空白.png"'}`
    - 这是正常现象，识别没有文字的空白图片时会出现这种结果。

    ### `200` 图片路径不存在

    - data 为字符串：`Image path dose not exist. Path:"图片路径".`
    - 例：`{'code':200,'data':'Image path dose not exist. Path: "D:\\不存在.png"'}`
    - 注意，在系统未开启utf-8支持（`使用 Unicode UTF-8 提供全球语言支持"`）时，不能读入含emoji等特殊字符的路径（如`😀.png`）。但一般的中文及其他 Unicode 字符路径是没问题的，不受系统区域及默认编码影响。

    ### `201` 图片路径string无法转换到wstring

    - data 为字符串：`Image path failed to convert to utf-16 wstring. Path: "图片路径".`
    - 使用API时，理论上不会报这个错。
    - 开发API时，若传入字符串的编码不合法，有可能报这个错。

    ### `202` 图片路径存在，但无法打开文件

    - data 为字符串：`Image open failed. Path: "图片路径".`
    - 可能由系统权限等原因引起。

    ### `203` 图片打开成功，但读取到的内容无法被opencv解码

    - data 为字符串：`Image decode failed. Path: "图片路径".`
    - 注意，引擎不以文件后缀来区分各种图片，而是对存在的路径，均读入字节尝试解码。若传入的文件路径不是图片，或图片已损坏，则会报这个错。
    - 反之，将正常图片的后缀改为别的（如`.png`改成`.jpg或.exe`），也可以被正常识别。

    ### `299` 未知异常

    - data 为字符串：`An unknown error has occurred.`
    - Please [open an issue](https://github.com/hiroi-sora/PaddleOCR-json/issues/new).
    */

    pub fn ocr<S: AsRef<str> + std::fmt::Display>(&mut self, image_path: S) -> IoResult<String> {
        self.write_fmt(format_args!("{}\n", &image_path))?;
        self.read_line()
    }

    /**
    OCRs the image in clipboard. Note that the returned JSON is not parsed or checked, and a valid JSON does not necessarily mean it is successful.

    # Examples

    ```no_run
    let mut p = paddleocr::Ppocr::new(std::path::PathBuf::from(".../PaddleOCR_json.exe",));
    println!("{}", p.ocr_clipboard().unwrap());
    ```

    # Results

    Apart from values in `ocr()`, `ocr_clipboard()` has its own clipboard-related errors.

    ## Return values

    根含两个元素：状态码 `code` 和内容 `data` 。

    状态码 `code` 为整数，每种状态码对应一种情况。

    ## Clipboard-specific code values

    ### `210` 剪贴板打开失败

    - data 为字符串：`Clipboard open failed.`
    - 可能由别的程序正在占用剪贴板等原因引起。

    ### `211` 剪贴板为空

    - data 为字符串：`Clipboard is empty.`

    ### `212` 剪贴板的格式不支持

    - data 为字符串：`Clipboard format is not valid.`
    - 引擎只能识别剪贴板中的位图或文件。若不是这两种格式（如复制了一段文本），则会报这个错。

    ### `213` 剪贴板获取内容句柄失败

    - data 为字符串：`Getting clipboard data handle failed.`
    - 可能由别的程序正在占用剪贴板等原因引起。

    ### `214` 剪贴板查询到的文件的数量不为1

    - data 为字符串：`Clipboard number of query files is not valid. Number: 文件数量`
    - 只允许一次复制一个文件。一次复制多个文件再调用OCR会得到此报错。

    ### `215` 剪贴板检索图形对象信息失败

    - data 为字符串：`Clipboard get bitmap object failed.`
    - 剪贴板中是位图，但获取位图信息失败。可能由别的程序正在占用剪贴板等原因引起。

    ### `216` 剪贴板获取位图数据失败

    - data 为字符串：`Getting clipboard bitmap bits failed.`
    - 剪贴板中是位图，获取位图信息成功，但读入缓冲区失败。可能由别的程序正在占用剪贴板等原因引起。

    ### `217` 剪贴板中位图的通道数不支持

    - data 为字符串：`Clipboard number of image channels is not valid. Number: 通道数`
    - 引擎只允许读入通道为1（黑白）、3（RGB）、4（RGBA）的图片。位图通道数不是1、3或4，会报这个错。

    ## Other code values

    ### `100` 识别到文字

    - data 为数组。数组每一项为字典，含三个元素：
    - `text`：文本内容，字符串。
    - `box`：文本包围盒，长度为4的数组，分别为左上角、右上角、右下角、左下角的`[x,y]`。整数。
    - `score`：识别置信度，浮点数。
    - 例：
    ```no_run
        {'code':100,'data':[{'box':[[13,5],[161,5],[161,27],[13,27]],'score':0.9996442794799805,'text':'飞舞的因果交流'}]}
    ```

    ### `101` 未识别到文字

    - data 为字符串：`No text found in image. Path:"图片路径"`
    - 例：`{'code':101,'data':'No text found in image. Path: "D:\\空白.png"'}`
    - 这是正常现象，识别没有文字的空白图片时会出现这种结果。

    ### `299` 未知异常

    - data 为字符串：`An unknown error has occurred.`
    - Please [open an issue](https://github.com/hiroi-sora/PaddleOCR-json/issues/new).
    */
    #[inline]
    pub fn ocr_clipboard(&mut self) -> IoResult<String> {
        self.ocr("clipboard")
    }

    #[cfg(feature = "parse")]
    pub fn ocr_and_parse<S: AsRef<str> + std::fmt::Display>(
        &mut self,
        image_path: S,
    ) -> Result<Vec<ContentData>, String> {
        let ocr_result = self.ocr(image_path);
        let Ok(ocr_string) = ocr_result.as_ref() else {
            return Err("OCR failed".to_string());
        };
        match serde_json::from_str::<OcrRec>(&ocr_string) {
            Ok(OcrRec::Content { data, .. }) => Ok(data),
            Ok(OcrRec::Message { code, data }) => Err(format!("{}: {}", code, data)),
            Err(e) => Err(format!("JSON parse failed: {}", e)),
        }
    }
}

impl Drop for Ppocr {
    fn drop(&mut self) {
        self.process.kill().err();
    }
}

#[cfg(test)]

mod tests {
    use crate::Ppocr;
    #[test]
    fn recognize() {
        let mut p = Ppocr::new(std::path::PathBuf::from(
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

    #[test]
    fn parse() {
        let mut p = Ppocr::new(std::path::PathBuf::from(
            "E:/code/paddleocr/pojnew/PaddleOCR_json.exe", // path to binary
        ))
        .unwrap(); // initialize

        // OCR files
        p.ocr_and_parse("C:/Users/Neko/Pictures/test1.png").unwrap();
    }
}
