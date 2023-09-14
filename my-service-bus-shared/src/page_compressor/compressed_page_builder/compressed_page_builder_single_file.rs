use std::io::Write;

use crate::{
    page_compressor::vec_writer::VecWriter,
    protobuf_models::{MessageProtobufModel, MessagesProtobufModel},
};

use super::CompressedPageWriterError;

pub struct CompressedPageBuilderSingleFile {
    messages: Option<Vec<MessageProtobufModel>>,
}

impl CompressedPageBuilderSingleFile {
    pub fn new() -> Self {
        let result = Self {
            messages: Some(Vec::new()),
        };

        result
    }

    pub fn add_message(&mut self, model: &MessageProtobufModel) {
        self.messages.as_mut().unwrap().push(model.clone());
    }

    pub fn get_payload(&mut self) -> Result<Vec<u8>, CompressedPageWriterError> {
        let messages = self.messages.take().unwrap();

        let messages = MessagesProtobufModel { messages };

        let mut payload = Vec::new();

        prost::Message::encode(&messages, &mut payload)?;

        let mut writer = VecWriter::new();

        {
            let mut zip = zip::ZipWriter::new(&mut writer);

            let options = zip::write::FileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);

            zip.start_file("d", options)?;

            let mut pos = 0;
            while pos < payload.len() {
                let size = zip.write(&payload[pos..])?;

                pos += size;
            }

            zip.finish()?;
        }

        Ok(writer.buf)
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use crate::page_compressor::CompressedPageReader;

    use super::*;

    #[test]
    fn test_compressed_as_single_file() {
        let mut builder = CompressedPageBuilderSingleFile::new();

        let msg1 = MessageProtobufModel::new(
            1.into(),
            DateTimeAsMicroseconds::now(),
            vec![0u8, 1u8, 2u8],
            vec![],
        );

        builder.add_message(&msg1);

        let msg2 = MessageProtobufModel::new(
            2.into(),
            DateTimeAsMicroseconds::now(),
            vec![3u8, 4u8, 5u8, 6u8],
            vec![],
        );

        builder.add_message(&msg2);

        let compressed = builder.get_payload().unwrap();

        let mut reader = CompressedPageReader::new(compressed).unwrap();

        assert_eq!(2, reader.get_messages_amount());

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
