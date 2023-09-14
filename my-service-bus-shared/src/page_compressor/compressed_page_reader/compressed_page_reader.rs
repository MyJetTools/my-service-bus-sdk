use rust_extensions::{AsSliceOrVec, SliceOrVecSeqReader};

use crate::protobuf_models::MessageProtobufModel;

use super::{
    CompressedPageReaderByFiles, CompressedPageReaderError, CompressedPageReaderSingleFile,
};

pub enum CompressedPageReader<'s> {
    ByFiles(CompressedPageReaderByFiles<'s>),
    SingleFile(CompressedPageReaderSingleFile),
}

impl<'s> CompressedPageReader<'s> {
    pub fn new(zipped: impl Into<AsSliceOrVec<'s, u8>>) -> Result<Self, CompressedPageReaderError> {
        let zipped: AsSliceOrVec<'_, u8> = zipped.into();

        let zipped: SliceOrVecSeqReader<'_, u8> = zipped.into();

        let mut file_reader = CompressedPageReaderByFiles::new(zipped)?;

        let decompress_as_single_file = file_reader.decompress_as_single_file()?;

        match decompress_as_single_file {
            Some(messages) => {
                let messages = messages.messages.into_iter().collect();
                Ok(Self::SingleFile(CompressedPageReaderSingleFile::new(
                    messages,
                )?))
            }

            None => Ok(Self::ByFiles(file_reader)),
        }
    }

    pub fn get_next_message(
        &mut self,
    ) -> Result<Option<MessageProtobufModel>, CompressedPageReaderError> {
        match self {
            CompressedPageReader::ByFiles(by_files) => by_files.get_next_message(),
            CompressedPageReader::SingleFile(by_single_file) => {
                let result = by_single_file.get_next_message();
                Ok(result)
            }
        }
    }

    pub fn get_files_amount(&self) -> usize {
        match self {
            CompressedPageReader::ByFiles(by_files) => by_files.get_files_amount(),
            CompressedPageReader::SingleFile(_) => 1,
        }
    }

    pub fn get_messages_amount(&self) -> usize {
        match self {
            CompressedPageReader::ByFiles(by_files) => by_files.get_files_amount(),
            CompressedPageReader::SingleFile(by_single_file) => {
                by_single_file.get_messages_amount()
            }
        }
    }
}
