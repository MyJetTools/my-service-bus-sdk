use super::SbMessageHeaders;

#[derive(Debug, Clone)]
pub struct MessageToPublish {
    pub headers: SbMessageHeaders,
    pub content: Vec<u8>,
}

impl MessageToPublish {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            headers: SbMessageHeaders::new(),
            content,
        }
    }

    pub fn new_with_headers(content: Vec<u8>, headers: SbMessageHeaders) -> Self {
        Self { headers, content }
    }
}
