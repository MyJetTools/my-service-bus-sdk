use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use rust_extensions::{Logger, StrOrString};

#[cfg(feature = "with-telemetry")]
use super::DeliveredMessageTelemetry;
use crate::{
    queue_with_intervals::QueueWithIntervals, MySbMessage, MyServiceBusSubscriberClient,
    MyServiceBusSubscriberClientCallback,
};

use super::{
    MessagesReader, MySbDeliveredMessage, MySbMessageDeserializer, SubscriberCallback,
    TopicQueueType,
};

pub struct SubscriberData {
    pub topic_id: StrOrString<'static>,
    pub queue_id: StrOrString<'static>,
    pub queue_type: TopicQueueType,
    pub logger: Arc<dyn Logger + Sync + Send + 'static>,
    pub client: Arc<dyn MyServiceBusSubscriberClient + Sync + Send + 'static>,
}

pub struct Subscriber<TMessageModel: MySbMessageDeserializer<Item = TMessageModel>> {
    data: Arc<SubscriberData>,
    pub callback: Arc<dyn SubscriberCallback<TMessageModel> + Sync + Send + 'static>,
}

impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static>
    Subscriber<TMessageModel>
{
    pub fn new(
        topic_id: StrOrString<'static>,
        queue_id: StrOrString<'static>,
        queue_type: TopicQueueType,
        callback: Arc<dyn SubscriberCallback<TMessageModel> + Sync + Send + 'static>,
        logger: Arc<dyn Logger + Sync + Send + 'static>,
        client: Arc<dyn MyServiceBusSubscriberClient + Sync + Send + 'static>,
    ) -> Self {
        let data = SubscriberData {
            topic_id,
            queue_id,
            queue_type,
            client,
            logger,
        };
        Self {
            callback,
            data: Arc::new(data),
        }
    }
}

#[async_trait::async_trait]
impl<TMessageModel: MySbMessageDeserializer<Item = TMessageModel> + Send + Sync + 'static>
    MyServiceBusSubscriberClientCallback for Subscriber<TMessageModel>
{
    fn get_topic_id(&self) -> &str {
        self.data.topic_id.as_str()
    }

    fn get_queue_id(&self) -> &str {
        self.data.queue_id.as_str()
    }
    fn get_queue_type(&self) -> TopicQueueType {
        self.data.queue_type
    }

    async fn new_events(
        &self,
        messages_to_deliver: Vec<MySbMessage>,
        confirmation_id: i64,
        connection_id: i32,
    ) {
        let mut messages = VecDeque::with_capacity(messages_to_deliver.len());

        let mut can_not_serialize_messages = QueueWithIntervals::new();

        let mut deserialize_error = None;

        for msg in messages_to_deliver {
            let content_result = TMessageModel::deserialize(&msg.content, &msg.headers);

            match content_result {
                Ok(contract) => {
                    #[cfg(feature = "with-telemetry")]
                    let my_telemetry = DeliveredMessageTelemetry::new(
                        self.get_topic_id(),
                        self.get_queue_id(),
                        msg.id,
                        &msg.headers,
                    );

                    let msg = MySbDeliveredMessage {
                        id: msg.id,
                        attempt_no: msg.attempt_no,
                        headers: msg.headers,
                        content: Some(contract),
                        raw: msg.content,
                        #[cfg(feature = "with-telemetry")]
                        my_telemetry: Some(my_telemetry),
                        inner: None,
                    };

                    messages.push_back(msg);
                }
                Err(err) => {
                    if deserialize_error.is_none() {
                        deserialize_error = Some(format!(
                            "Can not deserialize one of the messages. Err:{:?}",
                            err
                        ));
                    }
                    can_not_serialize_messages.enqueue(msg.id.get_value());
                }
            }
        }

        if messages.len() == 0 {
            self.data.client.confirm_delivery(
                self.data.topic_id.as_str(),
                self.data.queue_id.as_str(),
                confirmation_id,
                connection_id,
                true,
            );

            let mut ctx = HashMap::new();

            ctx.insert(
                "topicId".to_string(),
                self.data.topic_id.as_str().to_string(),
            );
            ctx.insert(
                "queueId".to_string(),
                self.data.queue_id.as_str().to_string(),
            );
            ctx.insert(
                "messages".to_string(),
                format!("{:?}", can_not_serialize_messages),
            );
            ctx.insert("confirmationId".to_string(), confirmation_id.to_string());

            self.data.logger.write_fatal_error(
                "new_events".to_string(),
                format!("Can not serialize messages"),
                Some(ctx),
            );
            return;
        }

        let reader = MessagesReader::new(
            self.data.clone(),
            messages,
            confirmation_id,
            connection_id,
            self.data.client.clone(),
        );

        let callback = self.callback.clone();

        tokio::spawn(async move {
            let mut reader = reader;

            if let Err(err) = callback.handle_messages(&mut reader).await {
                let mut ctx = HashMap::new();

                ctx.insert(
                    "topicId".to_string(),
                    reader.data.topic_id.as_str().to_string(),
                );
                ctx.insert(
                    "queueId".to_string(),
                    reader.data.queue_id.as_str().to_string(),
                );
                ctx.insert(
                    "confirmationId".to_string(),
                    reader.confirmation_id.to_string(),
                );
                reader.data.logger.write_fatal_error(
                    "new_events".to_string(),
                    format!("Can not handle messages. Err: {}", err.msg),
                    Some(ctx),
                );
            }
        });
    }
}
