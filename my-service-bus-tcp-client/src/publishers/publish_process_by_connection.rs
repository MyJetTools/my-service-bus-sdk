use std::{collections::HashMap, sync::Arc};

use my_service_bus_abstractions::PublishError;
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::TcpSocketConnection;
use rust_extensions::TaskCompletion;

pub struct PublishProcessByConnection {
    pub socket: Arc<TcpSocketConnection<TcpContract, MySbTcpSerializer>>,
    pub requests: HashMap<i64, TaskCompletion<(), PublishError>>,
}

impl PublishProcessByConnection {
    pub fn new(socket: Arc<TcpSocketConnection<TcpContract, MySbTcpSerializer>>) -> Self {
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
