#[cfg(feature = "with-telemetry")]
use super::DeliveredMessageTelemetry;
use crate::{MessageId, SbMessageHeaders};

use super::MySbMessageDeserializer;

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: SbMessageHeaders,
    pub raw: Vec<u8>,
    pub content: Option<TMessageModel>,
    #[cfg(feature = "with-telemetry")]
    pub my_telemetry: DeliveredMessageTelemetry,
}

impl<'s, TMessageModel: MySbMessageDeserializer<Item = TMessageModel>>
    MySbDeliveredMessage<TMessageModel>
{
    pub fn take_message(&mut self) -> TMessageModel {
        let result = self.content.take();
        if result.is_none() {
            panic!("Message was already taken");
        }

        return result.unwrap();
    }

    pub fn get_message(&self) -> &TMessageModel {
        if let Some(itm) = self.content.as_ref() {
            return itm;
        }
        panic!("Message was already taken");
    }
}
