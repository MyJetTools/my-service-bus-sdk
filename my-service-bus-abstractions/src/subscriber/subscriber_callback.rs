use crate::queue_with_intervals::QueueWithIntervals;

use super::{MessagesReader, MySbMessageDeserializer};

#[derive(Debug)]
pub enum MySbSubscriberHandleError {
    AllMessagesAreNotDelivered,
    TheMessagesAreNotDelivered(QueueWithIntervals),
    TheOnlyMessagesDelivered(QueueWithIntervals),
    Other(String),
}

#[async_trait::async_trait]
pub trait SubscriberCallback<
    TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static,
>
{
    async fn handle_messages(
        &self,
        messages_reader: &MessagesReader<TMessageModel>,
    ) -> Result<(), MySbSubscriberHandleError>;
}
