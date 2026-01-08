use crate::{publisher::MessageToPublish, subscriber::TopicQueueType, MySbMessage, PublishError};

#[cfg(feature = "with-telemetry")]
pub const MY_TELEMETRY_HEADER: &str = "process-id";

#[async_trait::async_trait]
pub trait MyServiceBusPublisherClient {
    async fn publish_message(
        &self,
        topic_id: &str,
        message: MessageToPublish,
        do_retry: bool,
    ) -> Result<(), PublishError>;

    async fn publish_messages(
        &self,
        topic_id: &str,
        message: &[MessageToPublish],
        do_retry: bool,
    ) -> Result<(), PublishError>;
}

pub trait MyServiceBusSubscriberClient {
    fn confirm_delivery(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        connection_id: i32,
        delivered: bool,
    );

    fn confirm_some_messages_ok(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        connection_id: i32,
        ok_messages: Vec<crate::queue_with_intervals::QueueIndexRange<i64>>,
    );

    fn intermediary_confirm(
        &self,
        topic_id: &str,
        queue_id: &str,
        confirmation_id: i64,
        connection_id: i32,
        ok_messages: Vec<crate::queue_with_intervals::QueueIndexRange<i64>>,
    );
}

#[async_trait::async_trait]
pub trait MyServiceBusSubscriberClientCallback {
    fn get_topic_id(&self) -> &str;
    fn get_queue_id(&self) -> &str;
    fn get_queue_type(&self) -> TopicQueueType;

    async fn new_events(
        &self,
        messages_to_deliver: Vec<MySbMessage>,
        confirmation_id: i64,
        connection_id: i32,
    );
}

pub trait GetMySbModelTopicId {
    fn get_topic_id() -> &'static str;
}
