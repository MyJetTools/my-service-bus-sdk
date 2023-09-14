use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(i64);

impl MessageId {
    pub fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn from_opt_i64(value: Option<i64>) -> Option<Self> {
        let value = value?;
        Some(Self(value))
    }

    pub fn get_value(&self) -> i64 {
        self.0
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }

    pub fn clone_with_delta(&self, delta: i64) -> Self {
        Self(self.0 + delta)
    }

    pub fn from_le_bytes(le_bytes: [u8; 8]) -> MessageId {
        MessageId::new(i64::from_le_bytes(le_bytes))
    }
}

impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<i64> for MessageId {
    fn as_ref(&self) -> &i64 {
        &self.0
    }
}

impl Into<MessageId> for i64 {
    fn into(self) -> MessageId {
        MessageId::new(self)
    }
}

impl<'s> Into<MessageId> for &'s i64 {
    fn into(self) -> MessageId {
        MessageId::new(*self)
    }
}

impl Into<i64> for MessageId {
    fn into(self) -> i64 {
        self.0
    }
}

pub trait AsMessageId {
    fn as_message_id(self) -> MessageId;
}

impl AsMessageId for i64 {
    fn as_message_id(self) -> MessageId {
        MessageId::new(self)
    }
}

impl AsMessageId for MessageId {
    fn as_message_id(self) -> MessageId {
        self
    }
}
