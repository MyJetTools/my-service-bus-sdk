pub mod new_messages_packet_builder;
pub mod tcp_contract_to_string;
pub mod tcp_message_id;
pub mod tcp_serializers;

mod my_sb_serializer_metadata;
mod new_messages_model;
mod packet_versions;
pub use new_messages_model::*;

mod tcp_contracts;
mod tcp_serializer;

pub use my_sb_serializer_metadata::*;

pub use packet_versions::PacketVersions;
pub use tcp_contracts::MySbTcpContract;
pub use tcp_serializer::MySbTcpSerializer;
mod tcp_protocol_version;
pub use tcp_protocol_version::*;

pub type MySbTcpConnection = my_tcp_sockets::tcp_connection::TcpSocketConnection<
    MySbTcpContract,
    MySbTcpSerializer,
    MySbSerializerState,
>;
