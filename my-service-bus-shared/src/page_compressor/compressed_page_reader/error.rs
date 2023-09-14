use std::num::ParseIntError;

use prost::DecodeError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum CompressedPageReaderError {
    ParseIntError(ParseIntError),
    ZipError(ZipError),
    InvalidSingleFileCompressedPage,
    DecodeError(DecodeError),
}

impl From<ZipError> for CompressedPageReaderError {
    fn from(src: ZipError) -> Self {
        Self::ZipError(src)
    }
}

impl From<ParseIntError> for CompressedPageReaderError {
    fn from(src: ParseIntError) -> Self {
        Self::ParseIntError(src)
    }
}

impl From<DecodeError> for CompressedPageReaderError {
    fn from(src: DecodeError) -> Self {
        Self::DecodeError(src)
    }
}
