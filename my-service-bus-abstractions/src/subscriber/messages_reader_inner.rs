use std::{collections::VecDeque, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{queue_with_intervals::QueueWithIntervals, MessageId};

use super::{MySbDeliveredMessage, MySbMessageDeserializer};

pub struct MessagesReaderInner<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub delivered: QueueWithIntervals,
    pub last_time_confirmation: DateTimeAsMicroseconds,
    /// How often the progress is sent to the broker as IntermediaryConfirm packet
    pub intermediary_confirmation_interval: Duration,
    pub current_message_id: Option<MessageId>,
    #[cfg(feature = "with-telemetry")]
    pub current_message_telemetry: Option<super::DeliveredMessageTelemetry>,
    pub messages: VecDeque<MySbDeliveredMessage<TMessageModel>>,
    pub force_all_failed: bool,
    pub total_messages_ids: QueueWithIntervals,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>>
    MessagesReaderInner<TMessageModel>
{
    pub fn new(messages: VecDeque<MySbDeliveredMessage<TMessageModel>>) -> Self {
        let mut total_messages_ids = QueueWithIntervals::new();
        for msg in messages.iter() {
            total_messages_ids.enqueue(msg.id.get_value());
        }
        Self {
            delivered: QueueWithIntervals::new(),
            last_time_confirmation: DateTimeAsMicroseconds::now(),
            intermediary_confirmation_interval:
                super::get_default_intermediary_confirmation_interval(),
            current_message_id: None,
            messages,
            #[cfg(feature = "with-telemetry")]
            current_message_telemetry: None,
            force_all_failed: false,
            total_messages_ids,
        }
    }

    pub fn set_current_message(
        &mut self,
        current_message_id: MessageId,
        #[cfg(feature = "with-telemetry")] current_message_telemetry: Option<
            super::DeliveredMessageTelemetry,
        >,
    ) {
        self.current_message_id = Some(current_message_id);
        #[cfg(feature = "with-telemetry")]
        {
            self.current_message_telemetry = current_message_telemetry;
        }
    }

    pub(crate) fn handled_message_id_as_ok(
        &mut self,
        message_id: MessageId,
        #[cfg(feature = "with-telemetry")] my_telemetry: Option<super::DeliveredMessageTelemetry>,
    ) {
        #[cfg(feature = "with-telemetry")]
        if let Some(mut my_telemetry) = my_telemetry {
            my_telemetry.enabled_duration_tracking_on_confirmation();
        }

        self.delivered.enqueue(message_id.get_value());
    }
}
