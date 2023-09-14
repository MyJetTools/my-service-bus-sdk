use prost::EncodeError;
use zip::result::ZipError;

#[derive(Debug)]
pub enum CompressedPageWriterError {
    ProtobufEncodeError(EncodeError),
    ZipError(ZipError),
    IoError(std::io::Error),
}

impl From<EncodeError> for CompressedPageWriterError {
    fn from(error: EncodeError) -> Self {
        Self::ProtobufEncodeError(error)
    }
}

impl From<ZipError> for CompressedPageWriterError {
    fn from(error: ZipError) -> Self {
        Self::ZipError(error)
    }
}

impl From<std::io::Error> for CompressedPageWriterError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
