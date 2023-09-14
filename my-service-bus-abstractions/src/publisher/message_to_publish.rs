use std::collections::HashMap;
#[derive(Debug, Clone)]
pub struct MessageToPublish {
    pub headers: Option<HashMap<String, String>>,
    pub content: Vec<u8>,
}

impl MessageToPublish {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            headers: None,
            content,
        }
    }

    pub fn new_with_headers(content: Vec<u8>, headers: HashMap<String, String>) -> Self {
        Self {
            headers: Some(headers),
            content,
        }
    }
}
