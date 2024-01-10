use std::collections::HashMap;

use my_service_bus_tcp_shared::{MySbSerializerMetadata, MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::TcpSocketConnection;

pub const PROTOCOL_VERSION: i32 = 3;

lazy_static::lazy_static! {
    pub static ref CLIENT_SERIALIZER_METADATA: MySbSerializerMetadata = {
        let mut attr = MySbSerializerMetadata::new(PROTOCOL_VERSION);

        attr.versions
            .set_packet_version(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

        attr
    };
}

pub async fn send_greeting(
    socket_ctx: &TcpSocketConnection<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
    app_name: &str,
    app_version: &str,
    client_version: &str,
) {
    let greeting = TcpContract::Greeting {
        name: format!("{}:{};{}", app_name, app_version, client_version),
        protocol_version: PROTOCOL_VERSION,
    };

    let mut payload = Vec::new();

    greeting.serialize(&mut payload, &CLIENT_SERIALIZER_METADATA);
    socket_ctx.send_bytes(payload.as_slice()).await;
}

pub async fn send_packet_versions(
    socket_ctx: &TcpSocketConnection<TcpContract, MySbTcpSerializer, MySbSerializerMetadata>,
) {
    let mut packet_versions = HashMap::new();
    packet_versions.insert(my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES, 1);

    let packet_versions = TcpContract::PacketVersions { packet_versions };

    let mut payload = Vec::new();
    packet_versions.serialize(&mut payload, &CLIENT_SERIALIZER_METADATA);

    socket_ctx.send_bytes(payload.as_slice()).await;
}
