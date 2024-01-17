use std::{collections::HashMap, sync::Arc};

use my_service_bus_abstractions::PublishError;
use my_service_bus_tcp_shared::MySbTcpConnection;

use rust_extensions::TaskCompletion;

pub struct PublishProcessByConnection {
    pub socket: Arc<MySbTcpConnection>,
    pub requests: HashMap<i64, TaskCompletion<(), PublishError>>,
}

impl PublishProcessByConnection {
    pub fn new(socket: Arc<MySbTcpConnection>) -> Self {
        Self {
            requests: HashMap::new(),
            socket,
        }
    }
}

impl Drop for PublishProcessByConnection {
    fn drop(&mut self) {
        for (_, mut task) in self.requests.drain() {
            task.set_error(PublishError::Disconnected);
        }
    }
}
