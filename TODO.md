## Review TODOs

- Ensure subscribers added after a connection is established are sent a `Subscribe` packet immediately (today they never get messages until reconnect).
- In `new_messages`, add a response path for unknown topic/queue (currently silently drops and never confirms/rejects, risking stalled deliveries).
- Replace panic on duplicate subscriber registration with a safe error/overwrite strategy.

