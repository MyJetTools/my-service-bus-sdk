# My Service Bus SDK – Quick Notes

This file captures the observed behavior and usage patterns from `my-service-bus-sdk` ([repo](https://github.com/MyJetTools/my-service-bus-sdk)).

## Dependencies
- `my-service-bus-tcp-client` (plus `my-service-bus-shared`)
- `tokio` with `full` features

## Settings
Implement `MyServiceBusSettings` to supply the connection string:
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

### Connection string

`get_host_port` returns a connection string. Two formats are accepted — the same shape
MyNoSqlServer uses:

| Value in settings | Host | Namespace |
|---|---|---|
| `127.0.0.1:6421` | `127.0.0.1:6421` | `default` |
| `host=127.0.0.1:6421` | `127.0.0.1:6421` | `default` |
| `host=127.0.0.1:6421;ns=alpha` | `127.0.0.1:6421` | `alpha` |

The new format is recognized **only** by the leading `host=`. Anything else is taken as a bare
host exactly as before — so existing settings files keep working untouched and nothing has to be
migrated.

```yaml
# legacy - still valid, works against any bus
MySb: 127.0.0.1:6421

# with a namespace
MySb: host=127.0.0.1:6421;ns=alpha
```

Rules for the new format:
- `;` separated `key=value` pairs; spaces around `;` and `=` are trimmed.
- Keys are case insensitive (`HOST=`/`NS=` are fine).
- `host` is mandatory; `ns` is optional.
- An unknown key (e.g. `namespace=` instead of `ns=`) is an **error** — better not to start at all
  than to silently publish into the wrong namespace.
- An invalid connection string panics on the connect attempt, with the reason in the message.

Namespace name rules: only `a-z`, `0-9` and `-`, does not start with `-`, length 1..=63.
Upper case is rejected rather than lower-cased — a typo has to fail loudly instead of creating
a garbage namespace.

The connection string is re-read before **every** connect attempt, so an application whose
settings are dynamic can change the namespace without a restart — it takes effect on the next
reconnect.

### Namespace on the wire

When a namespace is configured, the client sends a `SetNamespace` packet right after the
`Greeting` and before `PacketVersions` — the server has to fix the namespace before the first
publish or subscribe. It is fire-and-forget; the client does not wait for a reply.

The `default` namespace is **never** sent. A client configured with `ns=default` (or with no `ns`
at all) puts exactly the same bytes on the wire as before this feature existed, so it keeps
working against a bus that knows nothing about namespaces — such a node would drop the connection
on the unknown packet id.

Errors — an invalid name, or an attempt to change the namespace after the first publish/subscribe
— come back as the usual `Reject` packet and are written to the log.

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

### Long running handlers

The broker kicks the **whole connection** off if a subscriber stays in `OnDelivery` state longer
than its `delivery_timeout` (30 seconds by default). To avoid it the SDK pushes the current progress
as an `IntermediaryConfirm` packet — it confirms the messages handled so far and extends the timeout.

Nothing has to be done for the common case: on every `get_next_message()` the SDK checks how long
ago the previous packet was sent, and if it is more than 3 seconds — sends the current snapshot of
the delivered messages. Batches handled within milliseconds — the majority of them — never reach
the threshold and cost nothing: no extra packets, no background tasks.

The one case the SDK can not cover on its own is a **single** message handled longer than the broker
timeout: `get_next_message()` is simply not called during that time. Push the progress manually:

```rust
while let Some(msg) = messages_reader.get_next_message().await {
    // something long
    msg.mark_as_delivered().await;
    // confirms everything handled so far and resets the delivery deadline.
    // Fire and forget — the packet just goes into the outgoing buffer of the socket
    messages_reader.send_intermediary_confirmation().await;
}
```

The interval is configurable per reader with
`messages_reader.set_intermediary_confirmation_interval(Duration).await`, or process wide with
`my_service_bus_abstractions::subscriber::set_default_intermediary_confirmation_interval(Duration)`.
Keep it noticeably below the broker `delivery_timeout`.

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
