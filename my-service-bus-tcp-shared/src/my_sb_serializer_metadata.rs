use crate::{TcpContract, TcpProtocolVersion};

use super::PacketVersions;

use my_tcp_sockets::TcpSerializationMetadata;
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
    fn default() -> Self {
        Self {
            versions: PacketVersions::new(),
            tcp_protocol_version: 0i32.into(),
        }
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

impl TcpSerializationMetadata<TcpContract> for MySbSerializerMetadata {
    fn apply_tcp_contract(&mut self, contract: &TcpContract) {
        match contract {
            TcpContract::Greeting {
                name: _,
                protocol_version,
            } => {
                self.tcp_protocol_version = (*protocol_version).into();
            }
            TcpContract::PacketVersions { packet_versions } => {
                self.versions.update(packet_versions);
            }
            _ => {}
        }
    }
}
