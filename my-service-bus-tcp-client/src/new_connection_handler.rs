use std::collections::HashMap;

use my_service_bus_tcp_shared::{
    MySbSerializerState, MySbTcpContract, MySbTcpSerializer, DEFAULT_TCP_PROTOCOL_VERSION,
};
use my_tcp_sockets::tcp_connection::TcpSocketConnection;

pub async fn send_greeting(
    socket_ctx: &TcpSocketConnection<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>,
    app_name: &str,
    app_version: &str,
    client_version: &str,
) {
    let mut name = format!("{}:{};{}", app_name, app_version, client_version);

    if let Ok(value) = std::env::var("ENV_INFO") {
        name.push(';');
        name.push_str(&value);
    }

    let greeting = MySbTcpContract::Greeting {
        name,
        protocol_version: DEFAULT_TCP_PROTOCOL_VERSION,
    };

    socket_ctx.send(&greeting).await;
}

pub async fn send_packet_versions(
    socket_ctx: &TcpSocketConnection<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>,
) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    let packet_versions = MySbTcpContract::PacketVersions { packet_versions };

    socket_ctx.send(&packet_versions).await;
}
