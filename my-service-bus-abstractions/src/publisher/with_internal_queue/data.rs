use std::{collections::HashMap, sync::Arc};

use rust_extensions::auto_shrink::VecDequeAutoShrink;

use tokio::sync::Mutex;

use crate::MyServiceBusPublisherClient;

use super::super::MessageToPublish;

pub struct QueueToPublish {
    pub queue: VecDequeAutoShrink<MessageToPublish>,
    pub being_published: usize,
}

impl QueueToPublish {
    pub fn new() -> Self {
        Self {
            queue: VecDequeAutoShrink::new(32),
            being_published: 0,
        }
    }
}

pub struct PublisherWithInternalQueueData {
    pub topic_id: String,
    pub client: Arc<dyn MyServiceBusPublisherClient + Send + Sync + 'static>,
    pub queue_to_publish: Mutex<QueueToPublish>,

    pub logger: Arc<dyn rust_extensions::Logger + Send + Sync + 'static>,
}

impl PublisherWithInternalQueueData {
    pub async fn get_messages_to_publish(&self) -> Option<Vec<MessageToPublish>> {
        let mut write_access = self.queue_to_publish.lock().await;
        if write_access.queue.len() == 0 {
            return None;
        }

        let mut result = Vec::new();

        let mut size_to_publish = 0;

        while size_to_publish < 4_000_000 {
            if let Some(item) = write_access.queue.pop_front() {
                size_to_publish += item.content.len();
                result.push(item);
                write_access.being_published += 1;
            } else {
                break;
            }
        }

        Some(result)
    }

    pub async fn messages_are_published(&self) {
        let mut write_access = self.queue_to_publish.lock().await;
        write_access.being_published = 0;
    }

    pub async fn publish(&self, to_publish: &[MessageToPublish]) -> bool {
        let result = self
            .client
            .publish_messages(&self.topic_id, &to_publish, true)
            .await;

        match result {
            Ok(_) => return true,
            Err(err) => {
                let mut ctx = HashMap::new();
                ctx.insert("topicId".to_string(), self.topic_id.to_string());
                self.logger.write_fatal_error(
                    "publish".to_string(),
                    format!("Can not publish: Err: {:?}", err),
                    Some(ctx),
                );

                return false;
            }
        }
    }
}
