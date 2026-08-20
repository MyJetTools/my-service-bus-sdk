use std::collections::VecDeque;

use rust_extensions::SliceOrVec;

use crate::protobuf_models::{MessageProtobufModel, MessagesProtobufModel};

use super::CompressedPageReaderError;

pub struct CompressedPageReader {
    messages: VecDeque<MessageProtobufModel>,
    messages_amount: usize,
}

impl CompressedPageReader {
    pub fn new<'s>(
        compressed: impl Into<SliceOrVec<'s, u8>>,
    ) -> Result<Self, CompressedPageReaderError> {
        let compressed: SliceOrVec<'_, u8> = compressed.into();

        let payload = crate::page_compressor::decompress_payload(compressed.as_slice())?;

        let messages = MessagesProtobufModel::parse(payload.as_slice())?;

        let messages: VecDeque<MessageProtobufModel> = messages.messages.into_iter().collect();

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
