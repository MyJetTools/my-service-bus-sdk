#[cfg(feature = "with-telemetry")]
use my_telemetry::{EventDurationTracker, MyTelemetryContext};
use std::collections::HashMap;

use crate::MessageId;

use super::MySbMessageDeserializer;

pub struct MySbDeliveredMessage<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub id: MessageId,
    pub attempt_no: i32,
    pub headers: Option<HashMap<String, String>>,
    pub raw: Vec<u8>,
    pub content: Option<TMessageModel>,
    #[cfg(feature = "with-telemetry")]
    pub my_telemetry_ctx: Option<MyTelemetryContext>,
    #[cfg(feature = "with-telemetry")]
    pub event_tracker: Option<EventDurationTracker>,
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

    #[cfg(feature = "with-telemetry")]
    pub fn init_telemetry_context(&mut self, topic_id: &str, queue_id: &str) {
        use crate::MY_TELEMETRY_HEADER;

        if let Some(headers) = self.headers.as_ref() {
            if let Some(telemetry_value) = headers.get(MY_TELEMETRY_HEADER) {
                if let Ok(my_telemetry) = MyTelemetryContext::parse_from_string(telemetry_value) {
                    let event_duration_tracker = my_telemetry.start_event_tracking(format!(
                        "Handling event {}/{}. MsgId: {}",
                        topic_id,
                        queue_id,
                        self.id.get_value()
                    ));
                    self.my_telemetry_ctx = Some(my_telemetry);
                    self.event_tracker = Some(event_duration_tracker)
                }
            }
        }
    }

    #[cfg(feature = "with-telemetry")]
    pub fn take_or_create_telemetry(&mut self) -> MyTelemetryContext {
        if let Some(my_telemetry_ctx) = self.my_telemetry_ctx.take() {
            return my_telemetry_ctx;
        }

        MyTelemetryContext::new()
    }
}
