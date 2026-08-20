use crate::protobuf_models::{MessageProtobufModel, MessagesProtobufModel};

use super::CompressedPageWriterError;

/// Collects the messages of a page and emits them as a single compressed protobuf blob.
pub struct CompressedPageBuilder {
    messages: Vec<MessageProtobufModel>,
}

impl CompressedPageBuilder {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn add_message(&mut self, model: &MessageProtobufModel) {
        self.messages.push(model.clone());
    }

    pub fn get_payload(self) -> Result<Vec<u8>, CompressedPageWriterError> {
        let messages = MessagesProtobufModel {
            messages: self.messages,
        };

        let mut payload = Vec::new();

        messages.serialize(&mut payload)?;

        let result = crate::page_compressor::compress_payload(payload.as_slice())?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use crate::page_compressor::CompressedPageReader;

    use super::*;

    #[test]
    fn test_compressed_page() {
        let mut builder = CompressedPageBuilder::new();

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

        let result_msg = reader.get_next_message().unwrap();

        assert_eq!(
            msg1.get_message_id().get_value(),
            result_msg.get_message_id().get_value()
        );
        assert_eq!(msg1.data.as_slice(), result_msg.data.as_slice());

        let result_msg = reader.get_next_message().unwrap();

        assert_eq!(
            msg2.get_message_id().get_value(),
            result_msg.get_message_id().get_value()
        );
        assert_eq!(msg2.data.as_slice(), result_msg.data.as_slice());

        assert_eq!(true, reader.get_next_message().is_none());
    }

    #[test]
    fn test_empty_page() {
        let builder = CompressedPageBuilder::new();

        let compressed = builder.get_payload().unwrap();

        let mut reader = CompressedPageReader::new(compressed).unwrap();

        assert_eq!(0, reader.get_messages_amount());
        assert_eq!(true, reader.get_next_message().is_none());
    }
}
