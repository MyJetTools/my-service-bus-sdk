use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "with-telemetry")]
use my_telemetry::MyTelemetryContext;

use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

use crate::{MyServiceBusPublisherClient, PublishError};

use super::{
    super::{MessageToPublish, MySbMessageSerializer},
    PublisherWithInternalQueueData, QueueToPublish,
};

pub struct PublisherWithInternalQueue<TMessageModel: MySbMessageSerializer> {
    data: Arc<PublisherWithInternalQueueData>,
    event_sender: UnboundedSender<()>,
    pub item: Option<TMessageModel>,
}

impl<TMessageModel: MySbMessageSerializer> PublisherWithInternalQueue<TMessageModel> {
    pub fn new(
        topic_id: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        logger: Arc<dyn rust_extensions::Logger + Send + Sync + 'static>,
    ) -> Self {
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();

        let data = PublisherWithInternalQueueData {
            client,
            topic_id,
            queue_to_publish: Mutex::new(QueueToPublish::new()),
            logger,
        };

        let result = Self {
            event_sender,
            data: Arc::new(data),
            item: None,
        };

        let data = result.data.clone();
        tokio::spawn(events_publisher(data, event_receiver));

        result
    }

    pub async fn publish_and_forget(
        &self,
        message: TMessageModel,
        #[cfg(feature = "with-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), PublishError> {
        let result = message.serialize(None);

        if let Err(err) = result {
            return Err(PublishError::SerializationError(err));
        }

        #[cfg(not(feature = "with-telemetry"))]
        let (content, headers) = result.unwrap();

        #[cfg(feature = "with-telemetry")]
        let (content, mut headers) = result.unwrap();

        #[cfg(feature = "with-telemetry")]
        if let Some(my_telemetry) = telemetry_context.as_ref() {
            super::super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
        }

        let mut write_access = self.data.queue_to_publish.lock().await;
        write_access
            .queue
            .push_back(MessageToPublish { headers, content });

        if let Err(err) = self.event_sender.send(()) {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.data.topic_id.to_string());
            self.data.logger.write_error(
                "publish_and_forget".to_string(),
                format!("Can not publish message. Err: {}", err),
                Some(ctx),
            )
        }

        Ok(())
    }

    pub async fn publish_chunk_and_forget(
        &self,
        messages: Vec<TMessageModel>,
        #[cfg(feature = "with-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), PublishError> {
        let mut to_publish = Vec::with_capacity(messages.len());

        for message in messages {
            let result = message.serialize(None);

            if let Err(err) = result {
                return Err(PublishError::SerializationError(err));
            }

            #[cfg(not(feature = "with-telemetry"))]
            let (content, headers) = result.unwrap();

            #[cfg(feature = "with-telemetry")]
            let (content, mut headers) = result.unwrap();

            #[cfg(feature = "with-telemetry")]
            if let Some(my_telemetry) = telemetry_context.as_ref() {
                super::super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
            }

            let msg_to_publish = MessageToPublish { headers, content };
            to_publish.push(msg_to_publish);
        }

        let mut write_access = self.data.queue_to_publish.lock().await;
        for msg in to_publish {
            write_access.queue.push_back(msg);
        }

        if let Err(err) = self.event_sender.send(()) {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.data.topic_id.to_string());
            self.data.logger.write_error(
                "publish_and_forget".to_string(),
                format!("Can not publish message. Err: {}", err),
                Some(ctx),
            )
        }

        Ok(())
    }
    pub async fn get_queue_size(&self) -> usize {
        let read_access = self.data.queue_to_publish.lock().await;
        read_access.queue.len() + read_access.being_published
    }
}

async fn events_publisher(
    data: Arc<PublisherWithInternalQueueData>,
    mut event_receiver: UnboundedReceiver<()>,
) {
    let mut to_publish = None;
    loop {
        if to_publish.is_none() {
            tokio::sync::mpsc::UnboundedReceiver::recv(&mut event_receiver).await;
            to_publish = data.get_messages_to_publish().await;
        }

        if to_publish.is_none() {
            continue;
        }

        if data.publish(to_publish.as_ref().unwrap()).await {
            data.messages_are_published().await;
            to_publish = None;
        } else {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }
}
