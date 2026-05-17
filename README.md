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
- `client.get_publisher_with_internal_queue::<T>().await` returns `PublisherWithInternalQueue<T>`.
- `publish_and_forget` / `publish_chunk_and_forget` return immediately after enqueue — the actual send happens in the background.
- Use it when you don't need the publish result synchronously and want to avoid awaiting the network round-trip on the hot path.
- Errors returned from these methods are **serialization errors only**; transport failures are retried in the background.

```rust
let publisher = client.get_publisher_with_internal_queue::<MyContract>().await;
publisher.publish_and_forget(msg).await?;        // single message
publisher.publish_chunk_and_forget(msgs).await?; // batch
```

## Subscribers
```rust
client
    .subscribe::<MyContract>(
        "queue-id",
        true,  // auto_delete   — delete the queue when the subscriber disconnects
        false, // single_connection — only one active connection at a time
        Arc::new(MySubscriber {}),
    )
    .await;

#[async_trait::async_trait]
impl SubscriberCallback<MyContract> for MySubscriber {
    async fn handle_messages(
        &self,
        messages_reader: &MessagesReader<MyContract>,
    ) -> Result<(), MySbSubscriberHandleError> {
        while let Some(msg) = messages_reader.get_next_message().await {
            // handle msg — iterating with get_next_message marks the previous
            // message as delivered automatically
        }
        Ok(())
    }
}
```

### Delivery confirmation via `Result`

`handle_messages` returns `Result<(), MySbSubscriberHandleError>`. The variant
you return controls the final confirmation packet the SDK sends to the broker
when the `MessagesReader` is dropped:

| Return value | TCP confirmation sent to broker |
|---|---|
| `Ok(())` | Confirmation is derived from iteration state — `confirm_delivery(true)` if all messages were pulled via `get_next_message`, `confirm_some_messages_ok` for a partial set, `confirm_delivery(false)` if none. |
| `Err(AllMessagesAreNotDelivered)` | `AllMessagesConfirmedAsFail` — the whole batch is force-confirmed as not delivered, regardless of how far you iterated. |
| `Err(TheMessagesAreNotDelivered(qwi))` | `ConfirmSomeMessagesAsOk` with `total − qwi` as the delivered subset. If `qwi` covers all messages → `AllMessagesConfirmedAsFail`; if empty → `NewMessagesConfirmation`. |
| `Err(TheOnlyMessagesDelivered(qwi))` | `ConfirmSomeMessagesAsOk` with `qwi` as the delivered subset. Same edge cases as above. |
| `Err(Other(String))` | Writes a fatal log entry with the message; the broker confirmation falls back to the iteration-based path (same as `Ok`). Use this only for unexpected failures. |

`qwi` is a `my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals`
(default generic `<i64>`) built from `MessageId::get_value()`:

```rust
let mut failed = QueueWithIntervals::new();
failed.enqueue(msg.id.get_value());
return Err(MySbSubscriberHandleError::TheMessagesAreNotDelivered(failed));
```

For per-message control inside `Ok` flow, use `MySbDeliveredMessage::mark_as_not_delivered().await`
to drop the current message from the delivered set without bailing out of the whole batch.

## Ignore specific message
Set env var to skip delivery for a specific message:
```
SB_IGNORE_MESSAGE=TOPIC_ID=xxx;QUEUE_ID=xxx;MESSAGE_ID=xxx
```

## Operational notes
- Reconnect loop sleeps 1s while waiting for connection.
- Consider idempotent handlers; publisher retries can duplicate sends on reconnect.
- Queue flags recap (the two `subscribe` parameters combine into all 4 protocol variants):
  - `auto_delete = false`: durable queue persists.
  - `auto_delete = true`: ephemeral queue removed after a timeout on disconnect (short reconnects — e.g., within ~20s — keep the queue intact).
  - `single_connection = true`: a new connection will drop the previous one.
