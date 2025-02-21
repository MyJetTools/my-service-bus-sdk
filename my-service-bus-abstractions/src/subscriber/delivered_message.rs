use std::sync::Arc;

use tokio::sync::Mutex;

#[cfg(feature = "with-telemetry")]
use super::DeliveredMessageTelemetry;
use crate::{MessageId, SbMessageHeaders};

use super::{MessagesReaderInner, MySbMessageDeserializer};

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: SbMessageHeaders,
    pub raw: Vec<u8>,
    pub content: Option<TMessageModel>,
    #[cfg(feature = "with-telemetry")]
    pub my_telemetry: DeliveredMessageTelemetry,
    pub(crate) inner: Option<Arc<Mutex<MessagesReaderInner<TMessageModel>>>>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>>
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

    pub async fn mark_as_not_delivered(&self) {
        let inner = self.inner.as_ref().unwrap();
        let mut inner = inner.lock().await;

        inner.not_delivered.enqueue(self.id.get_value());
    }

    pub async fn mark_as_delivered(&mut self) {
        let inner = self.inner.as_ref().unwrap();
        let mut inner = inner.lock().await;
        inner.delivered.enqueue(self.id.get_value());
    }
}
