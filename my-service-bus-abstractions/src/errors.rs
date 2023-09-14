#[derive(Debug)]
pub enum PublishError {
    NoConnectionToPublish,
    SerializationError(String),
    Disconnected,
    Other(String),
}

impl Into<PublishError> for String {
    fn into(self) -> PublishError {
        PublishError::Other(self)
    }
}

#[derive(Debug)]
pub enum SubscriberError {
    CanNotDeserializeMessage(String),
}
