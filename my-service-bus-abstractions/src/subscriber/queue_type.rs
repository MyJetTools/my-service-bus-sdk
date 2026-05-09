#[derive(Debug, Clone, Copy)]
pub enum TopicQueueType {
    Permanent = 0,
    DeleteOnDisconnect = 1,
    PermanentWithSingleConnection = 2,
    DeleteOnDisconnectWithSingleConnection = 3,
}

impl TopicQueueType {
    pub fn from_u8(src: u8) -> TopicQueueType {
        match src {
            0 => TopicQueueType::Permanent,
            1 => TopicQueueType::DeleteOnDisconnect,
            2 => TopicQueueType::PermanentWithSingleConnection,
            3 => TopicQueueType::DeleteOnDisconnectWithSingleConnection,
            _ => TopicQueueType::DeleteOnDisconnect,
        }
    }

    pub fn into_u8(&self) -> u8 {
        *self as u8
    }

    pub fn from_flags(auto_delete: bool, single_connection: bool) -> TopicQueueType {
        match (auto_delete, single_connection) {
            (false, false) => TopicQueueType::Permanent,
            (true, false) => TopicQueueType::DeleteOnDisconnect,
            (false, true) => TopicQueueType::PermanentWithSingleConnection,
            (true, true) => TopicQueueType::DeleteOnDisconnectWithSingleConnection,
        }
    }
}
