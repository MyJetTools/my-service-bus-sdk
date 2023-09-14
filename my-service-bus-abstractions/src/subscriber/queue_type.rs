#[derive(Debug, Clone, Copy)]
pub enum TopicQueueType {
    Permanent = 0,
    DeleteOnDisconnect = 1,
    PermanentWithSingleConnection = 2,
}

impl TopicQueueType {
    pub fn from_u8(src: u8) -> TopicQueueType {
        match src {
            0 => TopicQueueType::Permanent,
            1 => TopicQueueType::DeleteOnDisconnect,
            2 => TopicQueueType::PermanentWithSingleConnection,
            _ => TopicQueueType::DeleteOnDisconnect,
        }
    }

    pub fn into_u8(&self) -> u8 {
        *self as u8
    }
}
