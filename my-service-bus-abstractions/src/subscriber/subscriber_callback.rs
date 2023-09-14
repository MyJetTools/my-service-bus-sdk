use super::{MessagesReader, MySbMessageDeserializer};

pub struct MySbSubscriberHandleError {
    pub msg: String,
}

#[async_trait::async_trait]
pub trait SubscriberCallback<
    TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static,
>
{
    async fn handle_messages(
        &self,
        messages_reader: &mut MessagesReader<TMessageModel>,
    ) -> Result<(), MySbSubscriberHandleError>;
}
