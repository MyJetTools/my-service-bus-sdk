use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use crate::{
    queue_with_intervals::QueueWithIntervals,
    subscriber::{MySbDeliveredMessage, MySbMessageDeserializer},
};

use super::SubscriberData;

pub struct MessagesReader<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    pub data: Arc<SubscriberData>,
    total_messages_amount: i64,
    messages: Option<VecDeque<MySbDeliveredMessage<TMessageModel>>>,
    pub confirmation_id: i64,
    delivered: QueueWithIntervals,
    connection_id: i32,
    current_message: Option<MySbDeliveredMessage<TMessageModel>>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> MessagesReader<TMessageModel> {
    pub fn new(
        data: Arc<SubscriberData>,
        messages: VecDeque<MySbDeliveredMessage<TMessageModel>>,
        confirmation_id: i64,
        connection_id: i32,
    ) -> Self {
        let total_messages_amount = messages.len() as i64;
        Self {
            data,
            messages: Some(messages),
            confirmation_id,
            delivered: QueueWithIntervals::new(),
            total_messages_amount,
            connection_id,
            current_message: None,
        }
    }

    fn handled_ok(&mut self, msg: &mut MySbDeliveredMessage<TMessageModel>) {
        #[cfg(feature = "with-telemetry")]
        if let Some(event_tracker) = msg.event_tracker.as_mut() {
            event_tracker.do_not_ignore_this_event();
        }
        self.delivered.enqueue(msg.id.get_value());
    }

    pub fn get_next_message<'s>(
        &'s mut self,
    ) -> Option<&'s mut MySbDeliveredMessage<TMessageModel>> {
        if let Some(mut message) = self.current_message.take() {
            self.handled_ok(&mut message);
        }

        let messages = self.messages.as_mut()?;
        self.current_message = Some(messages.pop_front()?);
        self.current_message.as_mut()
    }

    pub fn get_all(&mut self) -> Option<VecDeque<MySbDeliveredMessage<TMessageModel>>> {
        self.messages.take()
    }
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> Drop
    for MessagesReader<TMessageModel>
{
    fn drop(&mut self) {
        if self.delivered.len() == self.total_messages_amount {
            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                self.confirmation_id,
                self.connection_id,
                true,
            );
        } else if self.delivered.len() == 0 {
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
                    self.delivered.len(),
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
        };
    }
}
