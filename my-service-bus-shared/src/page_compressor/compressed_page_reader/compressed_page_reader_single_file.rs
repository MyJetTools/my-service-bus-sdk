use std::collections::VecDeque;

use zip::result::ZipError;

use crate::protobuf_models::MessageProtobufModel;

pub struct CompressedPageReaderSingleFile {
    messages: VecDeque<MessageProtobufModel>,
    messages_amount: usize,
}

impl CompressedPageReaderSingleFile {
    pub fn new(messages: VecDeque<MessageProtobufModel>) -> Result<Self, ZipError> {
        let messages_amount = messages.len();
        Ok(Self {
            messages,
            messages_amount,
        })
    }

    pub fn get_next_message(&mut self) -> Option<MessageProtobufModel> {
        self.messages.pop_front()
    }

    pub fn get_messages_amount(&self) -> usize {
        self.messages_amount
    }
}
