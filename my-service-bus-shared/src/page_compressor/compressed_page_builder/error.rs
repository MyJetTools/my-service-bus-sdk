use prost::EncodeError;

#[derive(Debug)]
pub enum CompressedPageWriterError {
    ProtobufEncodeError(EncodeError),
    IoError(std::io::Error),
}

impl From<EncodeError> for CompressedPageWriterError {
    fn from(error: EncodeError) -> Self {
        Self::ProtobufEncodeError(error)
    }
}

impl From<std::io::Error> for CompressedPageWriterError {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
