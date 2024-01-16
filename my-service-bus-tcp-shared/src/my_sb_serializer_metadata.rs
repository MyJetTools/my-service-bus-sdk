use my_tcp_sockets::{TcpSerializerMetadata, TcpSerializerMetadataFactory};

use crate::{MySbTcpContract, TcpProtocolVersion};

use super::PacketVersions;

pub const DEFAULT_TCP_PROTOCOL_VERSION: i32 = 3;
#[derive(Debug, Clone)]
pub struct PacketProtVer {
    pub packet_version: u8,
    pub tcp_protocol_version: TcpProtocolVersion,
}

#[derive(Clone)]
pub struct MySbSerializerMetadata {
    pub versions: PacketVersions,
    pub tcp_protocol_version: TcpProtocolVersion,
}

impl Default for MySbSerializerMetadata {
    #[cfg(not(feature = "tcp-client"))]
    fn default() -> Self {
        Self::new(0)
    }

    #[cfg(feature = "tcp-client")]
    fn default() -> Self {
        let mut result = Self::new(DEFAULT_TCP_PROTOCOL_VERSION);
        result
            .versions
            .set_packet_version(crate::tcp_message_id::NEW_MESSAGES, 1);
        result
    }
}

impl MySbSerializerMetadata {
    pub fn new(tcp_protocol_version: i32) -> Self {
        Self {
            versions: PacketVersions::new(),
            tcp_protocol_version: tcp_protocol_version.into(),
        }
    }

    pub fn get(&self, packet_no: u8) -> PacketProtVer {
        PacketProtVer {
            tcp_protocol_version: self.tcp_protocol_version.into(),
            packet_version: self.versions.get_packet_version(packet_no),
        }
    }
    pub fn get_packet_version(&self, packet_no: u8) -> u8 {
        self.versions.get_packet_version(packet_no)
    }
}

impl TcpSerializerMetadata<MySbTcpContract> for MySbSerializerMetadata {
    fn is_tcp_contract_related_to_metadata(&self, contract: &MySbTcpContract) -> bool {
        match contract {
            MySbTcpContract::Greeting {
                name: _,
                protocol_version: _,
            } => true,
            MySbTcpContract::PacketVersions { packet_versions: _ } => true,
            _ => false,
        }
    }

    fn apply_tcp_contract(&mut self, contract: &MySbTcpContract) {
        match contract {
            MySbTcpContract::Greeting {
                name: _,
                protocol_version,
            } => {
                self.tcp_protocol_version = (*protocol_version).into();
            }
            MySbTcpContract::PacketVersions { packet_versions } => {
                self.versions.update(packet_versions);
            }
            _ => {}
        }
    }
}

pub struct SbTcpSerializerMetadataFactory;
#[async_trait::async_trait]
impl TcpSerializerMetadataFactory<MySbTcpContract, MySbSerializerMetadata>
    for SbTcpSerializerMetadataFactory
{
    async fn create(&self) -> MySbSerializerMetadata {
        MySbSerializerMetadata::default()
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn metadata() {
        let a = std::mem::size_of::<super::MySbSerializerMetadata>();

        println!("Size of metadata: {}", a);
    }
}
