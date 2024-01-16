use std::sync::{atomic::AtomicBool, Arc};

use my_service_bus_tcp_shared::{MySbSerializerMetadata, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::TcpSocketConnection;
use rust_extensions::{Logger, StrOrString};

use crate::{publishers::MySbPublishers, subscribers::MySbSubscribers};

pub struct TcpClientData {
    pub app_name: StrOrString<'static>,
    pub app_version: StrOrString<'static>,
    pub client_version: String,
    pub publishers: Arc<MySbPublishers>,
    pub subscribers: Arc<MySbSubscribers>,
    pub logger: Arc<dyn Logger + Send + Sync + 'static>,
    pub has_connection: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl my_tcp_sockets::SocketEventCallback<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>
    for TcpClientData
{
    async fn connected(
        &self,
        connection: Arc<
            TcpSocketConnection<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
        >,
    ) {
        super::new_connection_handler::send_greeting(
            &connection,
            self.app_name.as_str(),
            self.app_version.as_str(),
            self.client_version.as_str(),
        )
        .await;

        super::new_connection_handler::send_packet_versions(&connection).await;

        self.publishers.new_connection(connection.clone()).await;
        self.subscribers.new_connection(connection.clone()).await;

        self.has_connection
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    async fn disconnected(
        &self,
        _connection: Arc<
            TcpSocketConnection<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
        >,
    ) {
        self.has_connection
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.publishers.disconnect().await;
        self.subscribers.disconnect().await;
    }

    async fn payload(
        &self,
        connection: &Arc<
            TcpSocketConnection<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
        >,
        contract: TcpContract,
    ) {
        match contract {
            my_service_bus_tcp_shared::TcpContract::PublishResponse { request_id } => {
                self.publishers.set_confirmed(request_id).await;
            }
            my_service_bus_tcp_shared::TcpContract::NewMessages {
                topic_id,
                queue_id,
                confirmation_id,
                messages,
            } => {
                self.subscribers
                    .new_messages(topic_id, queue_id, confirmation_id, connection.id, messages)
                    .await
            }
            _ => {}
        }
    }
}
