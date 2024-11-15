use std::io::Write;

use crate::{page_compressor::vec_writer::VecWriter, protobuf_models::MessageProtobufModel};

use super::CompressedPageWriterError;

pub struct CompressedPageBuilderByFiles {
    zip_writer: zip::ZipWriter<VecWriter>,
}

impl CompressedPageBuilderByFiles {
    pub fn new() -> Self {
        let result = Self {
            zip_writer: zip::ZipWriter::new(VecWriter::new()),
        };

        result
    }

    pub fn add_message(
        &mut self,
        model: &MessageProtobufModel,
    ) -> Result<(), CompressedPageWriterError> {
        let message_id = model.get_message_id();
        let file_name = format!("{}", message_id.get_value());

        let mut payload = Vec::new();

        model.serialize(&mut payload).unwrap();

        #[cfg(test)]
        println!("{}: {}", file_name, payload.len());

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        self.zip_writer.start_file(file_name, options)?;

        let mut pos = 0;
        while pos < payload.len() {
            let size = self.zip_writer.write(&payload[pos..])?;

            pos += size;
        }

        Ok(())
    }

    pub fn get_payload(self) -> Result<Vec<u8>, CompressedPageWriterError> {
        let result = self.zip_writer.finish()?;
        Ok(result.buf)
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use crate::page_compressor::CompressedPageReader;

    use super::*;

    #[test]
    fn test_compressed_by_files() {
        let mut builder = CompressedPageBuilderByFiles::new();

        let msg1 = MessageProtobufModel::new(
            1.into(),
            DateTimeAsMicroseconds::now(),
            vec![0u8, 1u8, 2u8],
            vec![],
        );

        builder.add_message(&msg1).unwrap();

        let msg2 = MessageProtobufModel::new(
            2.into(),
            DateTimeAsMicroseconds::now(),
            vec![3u8, 4u8, 5u8, 6u8],
            vec![],
        );

        builder.add_message(&msg2).unwrap();

        let compressed = builder.get_payload().unwrap();

        let mut reader = CompressedPageReader::new(compressed).unwrap();

        assert_eq!(2, reader.get_files_amount());

        let result_msg = reader.get_next_message().unwrap().unwrap();

        assert_eq!(
            msg1.get_message_id().get_value(),
            result_msg.get_message_id().get_value()
        );
        assert_eq!(msg1.data.as_slice(), result_msg.data.as_slice());

        let result_msg = reader.get_next_message().unwrap().unwrap();

        assert_eq!(
            msg2.get_message_id().get_value(),
            result_msg.get_message_id().get_value()
        );
        assert_eq!(msg2.data.as_slice(), result_msg.data.as_slice());

        let result_msg = reader.get_next_message().unwrap();

        assert_eq!(true, result_msg.is_none());
    }
}
