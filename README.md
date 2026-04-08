# My Service Bus SDK – Quick Notes

This file captures the observed behavior and usage patterns from `my-service-bus-sdk` ([repo](https://github.com/MyJetTools/my-service-bus-sdk)).

## Dependencies
- `my-service-bus-tcp-client` (plus `my-service-bus-shared`)
- `tokio` with `full` features

## Settings
Implement `MyServiceBusSettings` to supply the host/port:
```rust
#[derive(my_settings_reader::SettingsModel, Serialize, Deserialize, Debug, Clone)]
pub struct SettingsModel {
    #[serde(rename = "MySb")]
    pub my_sb: String,
}

#[async_trait::async_trait]
impl MyServiceBusSettings for SettingsReader {
    async fn get_host_port(&self) -> String {
        let read_access = self.settings.read().await;
        read_access.my_sb.clone()
    }
}
```

## Client creation
```rust
let client = MyServiceBusClient::new(
    "app-name",
    "app-version",
    settings_reader,          // Arc<dyn MyServiceBusSettings>
    logger_arc,
);
client.start().await;        // establish TCP connection and keep it alive
```

## Publishers

The SDK provides two publisher types for different use-cases:

### `MyServiceBusPublisher` – synchronous publish
- `get_publisher(do_retries: bool)` returns `MyServiceBusPublisher<T>`.
- Each `publish()` / `publish_messages()` call sends directly over the network and **awaits confirmation** before returning. The caller knows the result immediately.
- `do_retries = true` makes publish loop until connection is restored when errors are `NoConnectionToPublish`/`Disconnected`; `false` returns the error immediately.
- Serialization errors are not retried.
- The topic is created if missing (SDK sends `CreateTopicIfNotExists` during connection).

```rust
let publisher = client.get_publisher::<MyContract>(true).await;
publisher.publish(&msg, None).await?;
```

### `PublisherWithInternalQueue` – fire-and-forget with batching
- Messages are placed into an **internal queue** and the call returns immediately.
- A background task drains the queue in batches (up to 4 MB), sends each batch over the network, and waits for server confirmation (send-confirm pattern).
- While the current batch is in flight, **new messages continue to accumulate** in the queue without blocking the caller.
- On publish failure the batch is retried after a 3-second delay; the queue keeps accepting new messages during this time.

```rust
let publisher = PublisherWithInternalQueue::<MyContract>::new(topic_id, client, logger);
publisher.publish_and_forget(msg).await?;       // single message
publisher.publish_chunk_and_forget(msgs).await?; // batch
```

## Subscribers
```rust
client
    .subscribe::<MyContract>(
        "queue-id",
        TopicQueueType::DeleteOnDisconnect, // or PermanentWithSingleConnection, etc.
        Arc::new(MySubscriber {}),
    )
    .await;

#[async_trait::async_trait]
impl SubscriberCallback<MyContract> for MySubscriber {
    async fn new_events(&self, mut messages_reader: MessagesReader<MyContract>) {
        for msg in messages_reader.get_messages() {
            // handle message
            messages_reader.handled_ok(&msg);
        }
    }
}
```

## Ignore specific message
Set env var to skip delivery for a specific message:
```
SB_IGNORE_MESSAGE=TOPIC_ID=xxx;QUEUE_ID=xxx;MESSAGE_ID=xxx
```

## Operational notes
- Reconnect loop sleeps 1s while waiting for connection.
- Consider idempotent handlers; publisher retries can duplicate sends on reconnect.
- `TopicQueueType` recap:
  - `DeleteOnDisconnect`: ephemeral queue removed after a timeout on disconnect (short reconnects—e.g., within ~20s—keep the queue intact).
  - `Permanent`: durable queue persists.
  - `PermanentWithSingleConnection`: durable queue, and a new connection will drop the previous one.
