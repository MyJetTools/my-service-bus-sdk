use std::collections::HashMap;

use my_service_bus_tcp_shared::{
    MySbSerializerMetadata, MySbTcpContract, MySbTcpSerializer, DEFAULT_TCP_PROTOCOL_VERSION,
};
use my_tcp_sockets::tcp_connection::TcpSocketConnection;

pub async fn send_greeting(
    socket_ctx: &TcpSocketConnection<MySbTcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
    app_name: &str,
    app_version: &str,
    client_version: &str,
) {
    let greeting = MySbTcpContract::Greeting {
        name: format!("{}:{};{}", app_name, app_version, client_version),
        protocol_version: DEFAULT_TCP_PROTOCOL_VERSION,
    };

    socket_ctx.send(&greeting).await;
}

pub async fn send_packet_versions(
    socket_ctx: &TcpSocketConnection<MySbTcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    let packet_versions = MySbTcpContract::PacketVersions { packet_versions };

    socket_ctx.send(&packet_versions).await;
}
