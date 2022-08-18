use image::ImageError;
use serde_json::Error as JsonError;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum DScopeError {
    ExpectedDirectory { path: String },
    CannotReadFile { error: IoError, file: String },
    CannotWriteFile { error: IoError, file: String },
    CannotDecodeImage { error: ImageError, file: String },
    CannotDecodeInfo { error: JsonError, file: String },
}

impl std::fmt::Display for DScopeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DScopeError::ExpectedDirectory { path } => {
                f.write_fmt(format_args!("Expected directory: {}", path))
            }
            DScopeError::CannotReadFile { error, file } => {
                f.write_fmt(format_args!("Cannot read file {}: {}", file, error))
            }
            DScopeError::CannotWriteFile { error, file } => {
                f.write_fmt(format_args!("Cannot write file {}: {}", file, error))
            }
            DScopeError::CannotDecodeImage { error, file } => {
                f.write_fmt(format_args!("Cannot decode image {}: {}", file, error))
            }
            DScopeError::CannotDecodeInfo { error, file } => {
                f.write_fmt(format_args!("Cannot decode info {}: {}", file, error))
            }
        }
    }
}

impl std::error::Error for DScopeError {}

impl DScopeError {
    pub fn expected_directory(path: String) -> Self {
        Self::ExpectedDirectory { path }
    }
    pub fn cannot_read_file(error: IoError, file: String) -> Self {
        Self::CannotReadFile { error, file }
    }
    pub fn cannot_write_file(error: IoError, file: String) -> Self {
        Self::CannotWriteFile { error, file }
    }
    pub fn cannot_decode_image(error: ImageError, file: String) -> Self {
        Self::CannotDecodeImage { error, file }
    }
    pub fn cannot_decode_info(error: JsonError, file: String) -> Self {
        Self::CannotDecodeInfo { error, file }
    }
}

pub type DScopeResult<T> = Result<T, DScopeError>;
