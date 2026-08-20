use prost::DecodeError;

#[derive(Debug)]
pub enum CompressedPageReaderError {
    IoError(std::io::Error),
    DecodeError(DecodeError),
}

impl From<std::io::Error> for CompressedPageReaderError {
    fn from(src: std::io::Error) -> Self {
        Self::IoError(src)
    }
}

impl From<DecodeError> for CompressedPageReaderError {
    fn from(src: DecodeError) -> Self {
        Self::DecodeError(src)
    }
}
