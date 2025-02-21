use std::collections::VecDeque;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{queue_with_intervals::QueueWithIntervals, MessageId};

use super::{MySbDeliveredMessage, MySbMessageDeserializer};

pub struct MessagesReaderInner<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub delivered: QueueWithIntervals,
    pub not_delivered: QueueWithIntervals,
    pub prev_intermediary_confirmation_queue: QueueWithIntervals,
    pub last_time_confirmation: DateTimeAsMicroseconds,
    pub current_message_id: Option<MessageId>,
    pub messages: VecDeque<MySbDeliveredMessage<TMessageModel>>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>>
    MessagesReaderInner<TMessageModel>
{
    pub fn new(messages: VecDeque<MySbDeliveredMessage<TMessageModel>>) -> Self {
        Self {
            delivered: QueueWithIntervals::new(),
            not_delivered: QueueWithIntervals::new(),
            last_time_confirmation: DateTimeAsMicroseconds::now(),
            prev_intermediary_confirmation_queue: QueueWithIntervals::new(),
            current_message_id: None,
            messages,
        }
    }

    pub(crate) async fn handled_message_id_as_ok(&mut self, message_id: MessageId) {
        #[cfg(feature = "with-telemetry")]
        msg.my_telemetry.enabled_duration_tracking_on_confirmation();

        self.delivered.enqueue(message_id.get_value());
    }
}
