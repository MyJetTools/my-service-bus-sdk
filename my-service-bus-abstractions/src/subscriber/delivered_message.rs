use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{MessageId, SbMessageHeaders};

use super::{MessagesReaderInner, MySbMessageDeserializer};

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: SbMessageHeaders,
    pub raw: Vec<u8>,
    pub content: Option<TMessageModel>,
    #[cfg(feature = "with-telemetry")]
    pub(crate) my_telemetry: Option<super::DeliveredMessageTelemetry>,
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

        if let Some(message_id) = inner.current_message_id.take() {
            inner.not_delivered.enqueue(message_id.get_value());
        }
    }

    pub async fn mark_as_delivered(&self) {
        let inner = self.inner.as_ref().unwrap();
        let mut inner = inner.lock().await;
        if let Some(message_id) = inner.current_message_id.take() {
            inner.delivered.enqueue(message_id.get_value());
        }
    }
    #[cfg(feature = "with-telemetry")]
    pub async fn engage_telemetry(&self) -> my_telemetry::MyTelemetryContext {
        let inner = self.inner.as_ref().unwrap();
        let mut inner = inner.lock().await;

        if let Some(my_telemetry) = inner.current_message_telemetry.as_mut() {
            return my_telemetry.engage_telemetry();
        }

        my_telemetry::MyTelemetryContext::Empty
    }
}
