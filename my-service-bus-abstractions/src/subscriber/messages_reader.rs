use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{
    subscriber::{MySbDeliveredMessage, MySbMessageDeserializer},
    MyServiceBusSubscriberClient,
};

use super::{MessagesReaderInner, SubscriberData};

pub struct MessagesReader<
    TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static,
> {
    pub data: Arc<SubscriberData>,
    total_messages_amount: usize,

    pub confirmation_id: i64,
    inner: Arc<Mutex<MessagesReaderInner<TMessageModel>>>,
    connection_id: i32,
    intermediary_confirmation: Arc<dyn MyServiceBusSubscriberClient + Send + Sync + 'static>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static>
    MessagesReader<TMessageModel>
{
    pub fn new(
        data: Arc<SubscriberData>,
        messages: VecDeque<MySbDeliveredMessage<TMessageModel>>,
        confirmation_id: i64,
        connection_id: i32,
        intermediary_confirmation: Arc<dyn MyServiceBusSubscriberClient + Send + Sync + 'static>,
    ) -> Self {
        let total_messages_amount = messages.len();
        Self {
            data,
            confirmation_id,
            total_messages_amount,
            connection_id,
            inner: Arc::new(Mutex::new(MessagesReaderInner::new(messages))),
            intermediary_confirmation,
        }
    }

    pub async fn get_next_message(&self) -> Option<MySbDeliveredMessage<TMessageModel>> {
        let mut inner = self.inner.lock().await;

        if let Some(message_id) = inner.current_message_id.take() {
            #[cfg(feature = "with-telemetry")]
            let my_telemetry = inner.current_message_telemetry.take();
            inner.handled_message_id_as_ok(
                message_id,
                #[cfg(feature = "with-telemetry")]
                my_telemetry,
            );
        }

        let now = DateTimeAsMicroseconds::now();

        let last_confirmation_time = now - inner.last_time_confirmation;

        if last_confirmation_time.get_full_seconds() >= 5 {
            if inner.prev_intermediary_confirmation_queue.len() != inner.delivered.len() {
                self.intermediary_confirmation.intermediary_confirm(
                    self.data.topic_id.as_str(),
                    self.data.queue_id.as_str(),
                    self.confirmation_id,
                    self.connection_id,
                    inner.delivered.get_snapshot(),
                );

                inner.prev_intermediary_confirmation_queue = inner.delivered.clone();
                inner.last_time_confirmation = now;
            }
        }

        let mut next_message = inner.messages.pop_front()?;
        next_message.inner = self.inner.clone().into();
        inner.set_current_message(
            next_message.id,
            #[cfg(feature = "with-telemetry")]
            next_message.my_telemetry.take(),
        );

        Some(next_message)
    }

    /*
    pub fn get_all<'s>(
        &'s mut self,
    ) -> Option<std::collections::vec_deque::IterMut<'s, MySbDeliveredMessage<TMessageModel>>> {
        self.handle_current_messages_as_ok();

        let result = self.messages.take();

        let result = result?;

        self.current_message = CurrentMessage::Multiple(result);
        Some(self.current_message.unwrap_as_iterator())
    }
     */
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static> Drop
    for MessagesReader<TMessageModel>
{
    fn drop(&mut self) {
        let inner = self.inner.clone();
        let data = self.data.clone();

        let total_messages_amount = self.total_messages_amount;
        let confirmation_id = self.confirmation_id;
        let connection_id = self.connection_id;

        tokio::spawn(async move {
            let mut debug = false;
            if let Ok(debug_topic) = std::env::var("DEBUG_TOPIC") {
                if debug_topic == data.topic_id.as_str() {
                    debug = true;
                }
            };

            let inner = inner.lock().await;

            if debug {
                println!(
                    "Confirmation: Topic: {}, Queue:{}, Total Amount: {}, Delivered Amount: {},  Not Delivered amount: {}",
                    data.topic_id.as_str(),
                    data.queue_id.as_str(),
                    total_messages_amount,
                    inner.delivered.queue_size(),
                    inner.not_delivered.queue_size()
                );
            }

            if inner.delivered.queue_size() == total_messages_amount {
                data.client.confirm_delivery(
                    data.topic_id.as_str(),
                    data.queue_id.as_str(),
                    confirmation_id,
                    connection_id,
                    true,
                );

                if debug {
                    println!("All messages confirmed as Delivered")
                }
            } else if inner.delivered.queue_size() == 0 {
                let mut log_context = HashMap::new();
                log_context.insert("ConfirmationId".to_string(), confirmation_id.to_string());

                log_context.insert("TopicId".to_string(), data.topic_id.as_str().to_string());
                log_context.insert("QueueId".to_string(), data.queue_id.as_str().to_string());

                data.logger.write_error(
                    "Sending delivery confirmation".to_string(),
                    "All messages confirmed as fail".to_string(),
                    Some(log_context),
                );

                data.client.confirm_delivery(
                    data.topic_id.as_str(),
                    data.queue_id.as_str(),
                    confirmation_id,
                    connection_id,
                    false,
                );

                if debug {
                    println!("All messages confirmed as not Delivered")
                }
            } else {
                let mut log_context = HashMap::new();
                log_context.insert("ConfirmationId".to_string(), confirmation_id.to_string());

                log_context.insert("TopicId".to_string(), data.topic_id.as_str().to_string());
                log_context.insert("QueueId".to_string(), data.queue_id.as_str().to_string());

                data.logger.write_error(
                    "Sending delivery confirmation".to_string(),
                    format!(
                        "{} messages out of {} confirmed as Delivered",
                        inner.delivered.queue_size(),
                        total_messages_amount
                    ),
                    Some(log_context),
                );
                data.client.confirm_some_messages_ok(
                    data.topic_id.as_str(),
                    data.queue_id.as_str(),
                    confirmation_id,
                    connection_id,
                    inner.delivered.get_snapshot(),
                );

                if debug {
                    println!(
                        "Some messages {:?} confirmed as not Delivered",
                        inner.delivered.get_snapshot()
                    )
                }
            };
        });
    }
}
