use my_service_bus_abstractions::MessageId;
use prost::{DecodeError, EncodeError};
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageProtobufModel {
    #[prost(int64, tag = "1")]
    message_id: i64,
    #[prost(int64, tag = "2")]
    created: i64,
    #[prost(bytes, tag = "3")]
    pub data: Vec<u8>,
    #[prost(message, repeated, tag = "4")]
    pub headers: Vec<MessageMetaDataProtobufModel>,
}

impl MessageProtobufModel {
    pub fn new(
        message_id: MessageId,
        create: DateTimeAsMicroseconds,
        data: Vec<u8>,
        headers: Vec<MessageMetaDataProtobufModel>,
    ) -> Self {
        Self {
            message_id: message_id.get_value(),
            created: create.unix_microseconds,
            data,
            headers,
        }
    }

    pub fn parse(payload: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(payload)
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) -> Result<(), EncodeError> {
        prost::Message::encode(self, dest)
    }

    pub fn get_message_id(&self) -> MessageId {
        self.message_id.into()
    }

    pub fn get_created(&self) -> DateTimeAsMicroseconds {
        DateTimeAsMicroseconds::new(self.created)
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessagesProtobufModel {
    #[prost(message, repeated, tag = "1")]
    pub messages: Vec<MessageProtobufModel>,
}

impl MessagesProtobufModel {
    pub fn parse(payload: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(payload)
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) -> Result<(), EncodeError> {
        prost::Message::encode(self, dest)
    }
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MessageMetaDataProtobufModel {
    #[prost(string, tag = "1")]
    pub key: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub value: ::prost::alloc::string::String,
}
