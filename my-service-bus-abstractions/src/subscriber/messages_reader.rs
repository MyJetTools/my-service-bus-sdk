use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    queue_with_intervals::QueueWithIntervals,
    subscriber::{MySbDeliveredMessage, MySbMessageDeserializer},
    MessageId,
};

use super::{CurrentMessage, SubscriberData};

pub struct MessagesReader<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub data: Arc<SubscriberData>,
    total_messages_amount: usize,
    messages: Option<VecDeque<MySbDeliveredMessage<TMessageModel>>>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    not_delivered: QueueWithIntervals,
    connection_id: i32,
    current_message: CurrentMessage<TMessageModel>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> MessagesReader<TMessageModel> {
    pub fn new(
        data: Arc<SubscriberData>,
        messages: VecDeque<MySbDeliveredMessage<TMessageModel>>,
        confirmation_id: i64,
        connection_id: i32,
    ) -> Self {
        let total_messages_amount = messages.len();
        Self {
            data,
            messages: Some(messages),
            confirmation_id,
            delivered: QueueWithIntervals::new(),
            total_messages_amount,
            connection_id,
            current_message: CurrentMessage::None,
            not_delivered: QueueWithIntervals::new(),
        }
    }

    fn handled_ok(&mut self, msg: &mut MySbDeliveredMessage<TMessageModel>) {
        #[cfg(feature = "with-telemetry")]
        msg.my_telemetry.enabled_duration_tracking_on_confirmation();

        if !self.not_delivered.has_message(msg.id.get_value()) {
            self.delivered.enqueue(msg.id.get_value());
        }
    }

    fn handle_current_messages_as_ok(&mut self) {
        match self.current_message.take() {
            CurrentMessage::Single(mut msg) => self.handled_ok(&mut msg),
            CurrentMessage::Multiple(msgs) => {
                for mut msg in msgs {
                    self.handled_ok(&mut msg)
                }
            }
            CurrentMessage::None => {}
        }
    }

    pub fn get_next_message<'s>(
        &'s mut self,
    ) -> Option<&'s mut MySbDeliveredMessage<TMessageModel>> {
        self.handle_current_messages_as_ok();

        let messages = self.messages.as_mut()?;
        let next_message = messages.pop_front()?;
        self.current_message = CurrentMessage::Single(next_message);
        Some(self.current_message.unwrap_as_single_message_mut())
    }

    pub fn mark_as_not_delivered(&mut self, message_id: MessageId) {
        let message_id = message_id.get_value();
        self.not_delivered.enqueue(message_id);
        let _ = self.delivered.remove(message_id);
    }

    pub fn get_all<'s>(
        &'s mut self,
    ) -> Option<std::collections::vec_deque::IterMut<'s, MySbDeliveredMessage<TMessageModel>>> {
        self.handle_current_messages_as_ok();

        let result = self.messages.take();

        let result = result?;

        self.current_message = CurrentMessage::Multiple(result);
        Some(self.current_message.unwrap_as_iterator())
    }
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> Drop
    for MessagesReader<TMessageModel>
{
    fn drop(&mut self) {
        let mut debug = false;
        if let Ok(debug_topic) = std::env::var("DEBUG_TOPIC") {
            if debug_topic == self.data.topic_id.as_str() {
                debug = true;
            }
        };

        if debug {
            println!(
                "Confirmation: Topic: {}, Queue:{}, Total Amount: {}, Delivered Amount: {},  Not Delivered amount: {}",
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.total_messages_amount,
                self.delivered.queue_size(),
                self.not_delivered.queue_size()
            );
        }
        if self.delivered.queue_size() == self.total_messages_amount {
            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                true,
            );

            if debug {
                println!("All messages confirmed as Delivered")
            }
        } else if self.delivered.queue_size() == 0 {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert(
                "TopicId".to_string(),
                self.data.topic_id.as_str().to_string(),
            );
            log_context.insert(
                "QueueId".to_string(),
                self.data.queue_id.as_str().to_string(),
            );

            self.data.logger.write_error(
                "Sending delivery confirmation".to_string(),
                "All messages confirmed as fail".to_string(),
                Some(log_context),
            );

            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                false,
            );

            if debug {
                println!("All messages confirmed as not Delivered")
            }
        } else {
            let mut log_context = HashMap::new();
            log_context.insert(
                "ConfirmationId".to_string(),
                self.confirmation_id.to_string(),
            );

            log_context.insert(
                "TopicId".to_string(),
                self.data.topic_id.as_str().to_string(),
            );
            log_context.insert(
                "QueueId".to_string(),
                self.data.queue_id.as_str().to_string(),
            );

            self.data.logger.write_error(
                "Sending delivery confirmation".to_string(),
                format!(
                    "{} messages out of {} confirmed as Delivered",
                    self.delivered.queue_size(),
                    self.total_messages_amount
                ),
                Some(log_context),
            );
            self.data.client.confirm_some_messages_ok(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                self.delivered.get_snapshot(),
            );

            if debug {
                println!(
                    "Some messages {:?} confirmed as not Delivered",
                    self.delivered.get_snapshot()
                )
            }
        };
    }
}
