use crate::{SbMessageHeaders, SubscriberError};

pub trait MySbMessageDeserializer {
    type Item;
    fn deserialize(src: &[u8], headers: &SbMessageHeaders) -> Result<Self::Item, SubscriberError>;
}
