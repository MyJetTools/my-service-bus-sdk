use crate::protobuf_models::MessageProtobufModel;

use super::{
    CompressedPageBuilderByFiles, CompressedPageBuilderSingleFile, CompressedPageWriterError,
};

pub enum CompressedPageBuilder {
    SingleFile(CompressedPageBuilderSingleFile),
    ByFiles(CompressedPageBuilderByFiles),
}

impl CompressedPageBuilder {
    pub fn new_as_single_file() -> Self {
        Self::SingleFile(CompressedPageBuilderSingleFile::new())
    }

    pub fn new_by_files() -> Self {
        Self::ByFiles(CompressedPageBuilderByFiles::new())
    }

    pub fn add_message(
        &mut self,
        model: &MessageProtobufModel,
    ) -> Result<(), CompressedPageWriterError> {
        match self {
            CompressedPageBuilder::SingleFile(single_file) => {
                single_file.add_message(model);
                Ok(())
            }
            CompressedPageBuilder::ByFiles(by_files) => by_files.add_message(model),
        }
    }

    pub fn get_payload(&mut self) -> Result<Vec<u8>, CompressedPageWriterError> {
        match self {
            CompressedPageBuilder::SingleFile(single_file) => single_file.get_payload(),
            CompressedPageBuilder::ByFiles(by_files) => by_files.get_payload(),
        }
    }
}
