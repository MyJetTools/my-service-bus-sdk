use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::Duration,
};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{
    queue_with_intervals::QueueWithIntervals,
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

        let since_last_confirmation = now
            .duration_since(inner.last_time_confirmation)
            .as_positive_or_zero();

        // Getting here means the previous message is handled - so there is always something
        // to push. Handling of the batch is slow enough to bump into the interval - let the broker
        // know about the progress. It confirms the messages handled so far and - since the broker
        // resets the delivery deadline on IntermediaryConfirm - keeps the connection alive
        if since_last_confirmation >= inner.intermediary_confirmation_interval {
            send_intermediary_confirm_packet(
                self.intermediary_confirmation.as_ref(),
                self.data.as_ref(),
                self.confirmation_id,
                self.connection_id,
                &mut inner,
                now,
            );
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

    /// Tells the broker "I am still alive" and pushes whatever is delivered by now.
    /// Nothing is checked - neither how long ago the previous packet was sent, nor whether
    /// anything has been delivered since then. An empty delivered set is a valid packet as well:
    /// the broker takes it, confirms nothing and just resets the delivery deadline.
    ///
    /// Two things happen on the broker side:
    /// * messages which are already marked as delivered are confirmed - they are not going
    ///   to be redelivered even if the rest of the batch fails;
    /// * the delivery deadline is reset - the broker does not kick the connection off.
    ///
    /// Fire and forget - nothing is awaited from the broker. The packet is serialized into
    /// the outgoing buffer of the socket right here, so the only thing the caller awaits
    /// is the lock of the reader, which is free in the vast majority of the cases.
    ///
    /// get_next_message() pushes the progress on its own, so this method is needed when the loop
    /// stays inside a single message longer than delivery_timeout on the broker side - in this case
    /// get_next_message() is simply not called during that time. Call it from the middle of such
    /// a handler, preferably right after message.mark_as_delivered().
    pub async fn send_intermediary_confirm(&self) {
        let mut inner = self.inner.lock().await;

        send_intermediary_confirm_packet(
            self.intermediary_confirmation.as_ref(),
            self.data.as_ref(),
            self.confirmation_id,
            self.connection_id,
            &mut inner,
            DateTimeAsMicroseconds::now(),
        );
    }

    /// How often get_next_message() pushes the progress to the broker. Default value is 3 seconds.
    /// It has to be noticeably less than delivery_timeout on the broker side (30 seconds by default)
    pub async fn set_intermediary_confirmation_interval(&self, interval: Duration) {
        let mut inner = self.inner.lock().await;
        inner.intermediary_confirmation_interval = interval;
    }

    pub async fn mark_all_failed(&self) {
        let mut inner = self.inner.lock().await;
        inner.force_all_failed = true;
    }

    pub async fn mark_messages_failed(&self, failed: QueueWithIntervals) {
        let mut inner = self.inner.lock().await;
        let mut delivered = inner.total_messages_ids.clone();
        for id in failed.iter() {
            let _ = delivered.remove(id);
        }
        inner.delivered = delivered;
        inner.current_message_id = None;
        inner.force_all_failed = false;
    }

    pub async fn mark_only_these_delivered(&self, delivered: QueueWithIntervals) {
        let mut inner = self.inner.lock().await;
        inner.delivered = delivered;
        inner.current_message_id = None;
        inner.force_all_failed = false;
    }
}

fn send_intermediary_confirm_packet<
    TMessageModel: MySbMessageDeserializer<Item = TMessageModel>,
>(
    client: &(dyn MyServiceBusSubscriberClient + Send + Sync + 'static),
    data: &SubscriberData,
    confirmation_id: i64,
    connection_id: i32,
    inner: &mut MessagesReaderInner<TMessageModel>,
    now: DateTimeAsMicroseconds,
) {
    client.intermediary_confirm(
        data.topic_id.as_str(),
        data.queue_id.as_str(),
        confirmation_id,
        connection_id,
        inner.delivered.get_snapshot(),
    );

    inner.last_time_confirmation = now;
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
                    "Confirmation: Topic: {}, Queue:{}, Total Amount: {}, Delivered Amount: {}",
                    data.topic_id.as_str(),
                    data.queue_id.as_str(),
                    total_messages_amount,
                    inner.delivered.queue_size(),
                );
            }

            if inner.force_all_failed {
                let mut log_context = HashMap::new();
                log_context.insert("ConfirmationId".to_string(), confirmation_id.to_string());
                log_context.insert("TopicId".to_string(), data.topic_id.as_str().to_string());
                log_context.insert("QueueId".to_string(), data.queue_id.as_str().to_string());

                data.logger.write_error(
                    "Sending delivery confirmation".to_string(),
                    "Subscriber returned AllMessagesAreNotDelivered — confirming all messages as fail"
                        .to_string(),
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
                    println!("Forced: all messages confirmed as not Delivered")
                }
            } else if inner.delivered.queue_size() == total_messages_amount {
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
