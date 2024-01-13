use crate::{MessageId, SbMessageHeaders};

pub trait MyServiceBusMessage {
    fn get_id(&self) -> MessageId;
    fn get_attempt_no(&self) -> i32;
    fn get_headers(&self) -> &SbMessageHeaders;
    fn get_content(&self) -> &[u8];
}

#[derive(Debug, Clone)]
pub struct MySbMessage {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: SbMessageHeaders,
    pub content: Vec<u8>,
}

impl MyServiceBusMessage for MySbMessage {
    fn get_id(&self) -> MessageId {
        self.id
    }

    fn get_attempt_no(&self) -> i32 {
        self.attempt_no
    }

    fn get_headers(&self) -> &SbMessageHeaders {
        &self.headers
    }

    fn get_content(&self) -> &[u8] {
        &self.content
    }
}
