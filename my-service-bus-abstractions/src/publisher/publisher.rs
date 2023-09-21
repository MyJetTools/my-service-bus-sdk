use std::{collections::HashMap, sync::Arc};

#[cfg(feature = "with-telemetry")]
use my_telemetry::MyTelemetryContext;
use rust_extensions::Logger;

use crate::{MyServiceBusPublisherClient, PublishError};

use super::{MessageToPublish, MySbMessageSerializer};

pub struct MyServiceBusPublisher<TMessageModel: MySbMessageSerializer> {
    pub topic_id: String,
    pub client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
    pub do_retries: bool,
    pub itm: Option<TMessageModel>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
}

impl<TMessageModel: MySbMessageSerializer> MyServiceBusPublisher<TMessageModel> {
    pub fn new(
        topic_id: String,
        client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
        do_retries: bool,
        logger: Arc<dyn Logger + Send + Sync + 'static>,
    ) -> Self {
        Self {
            topic_id,
            client,
            do_retries,
            logger,
            itm: None,
        }
    }

    pub async fn publish(
        &self,
        message: &TMessageModel,
        #[cfg(feature = "with-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), PublishError> {
        let content = message.serialize(None);

        if let Err(err) = content {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger
                .write_fatal_error("publish".to_string(), err.clone(), Some(ctx));
            return Err(PublishError::SerializationError(err));
        }

        #[cfg(not(feature = "with-telemetry"))]
        let (content, headers) = content.unwrap();

        #[cfg(feature = "with-telemetry")]
        let (content, mut headers) = content.unwrap();

        #[cfg(feature = "with-telemetry")]
        if let Some(my_telemetry) = telemetry_context.as_ref() {
            super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
        }

        let result = self
            .client
            .publish_message(
                &self.topic_id,
                MessageToPublish { headers, content },
                self.do_retries,
            )
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    pub async fn publish_with_headers(
        &self,
        message: &TMessageModel,
        headers: HashMap<String, String>,
        #[cfg(feature = "with-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), PublishError> {
        let content = message.serialize(Some(headers));

        if let Err(err) = content {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_fatal_error(
                "publish_with_headers".to_string(),
                err.clone(),
                Some(ctx),
            );

            return Err(PublishError::SerializationError(err));
        }

        #[cfg(not(feature = "with-telemetry"))]
        let (content, headers) = content.unwrap();

        #[cfg(feature = "with-telemetry")]
        let (content, mut headers) = content.unwrap();

        #[cfg(feature = "with-telemetry")]
        if let Some(my_telemetry) = telemetry_context.as_ref() {
            super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
        }

        let result = self
            .client
            .publish_message(
                &self.topic_id,
                MessageToPublish { headers, content },
                self.do_retries,
            )
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    #[cfg(not(feature = "with-telemetry"))]
    pub async fn publish_messages(&self, messages: &[TMessageModel]) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for message in messages {
            let content = message.serialize(None);

            if let Err(err) = content {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish_messages".to_string(),
                    err.clone(),
                    Some(ctx),
                );

                return Err(PublishError::SerializationError(err));
            }

            let (content, headers) = content.unwrap();
            messages_to_publish.push(MessageToPublish { headers, content });
        }

        let result = self
            .client
            .publish_messages(&self.topic_id, &messages_to_publish, self.do_retries)
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }
    #[cfg(feature = "with-telemetry")]
    pub async fn publish_messages<'s>(
        &'s self,
        messages: impl Iterator<Item = (&'s TMessageModel, Option<&MyTelemetryContext>)>,
    ) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::new();

        for (message, telemetry_context) in messages {
            let content = message.serialize(None);

            if let Err(err) = content {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish_messages".to_string(),
                    err.clone(),
                    Some(ctx),
                );

                return Err(PublishError::SerializationError(err));
            }

            let (content, mut headers) = content.unwrap();

            if let Some(my_telemetry) = telemetry_context.as_ref() {
                super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
            }

            messages_to_publish.push(MessageToPublish { headers, content });
        }

        let result = self
            .client
            .publish_messages(&self.topic_id, &messages_to_publish, self.do_retries)
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }

    pub async fn publish_messages_with_header(
        &self,
        messages: Vec<(TMessageModel, Option<HashMap<String, String>>)>,
        #[cfg(feature = "with-telemetry")] telemetry_context: Option<&MyTelemetryContext>,
    ) -> Result<(), PublishError> {
        let mut messages_to_publish = Vec::with_capacity(messages.len());

        for (contract, headers) in messages {
            let content = contract.serialize(headers);

            if let Err(err) = content {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish_messages_with_header".to_string(),
                    err.clone(),
                    Some(ctx),
                );

                return Err(PublishError::SerializationError(err));
            }

            #[cfg(not(feature = "with-telemetry"))]
            let (content, headers) = content.unwrap();

            #[cfg(feature = "with-telemetry")]
            let (content, mut headers) = content.unwrap();

            #[cfg(feature = "with-telemetry")]
            if let Some(my_telemetry) = telemetry_context.as_ref() {
                super::my_telemetry::apply_publish_telemetry(&mut headers, my_telemetry)
            }

            messages_to_publish.push(MessageToPublish { content, headers });
        }

        let result = self
            .client
            .publish_messages(&self.topic_id, &messages_to_publish, self.do_retries)
            .await;

        if let Err(err) = &result {
            let mut ctx = HashMap::new();
            ctx.insert("topicId".to_string(), self.topic_id.to_string());
            self.logger.write_error(
                "publish_messages_with_header".to_string(),
                format!("Can not publish message. Error: {:?}", err),
                Some(ctx),
            );
        }

        result
    }
}
