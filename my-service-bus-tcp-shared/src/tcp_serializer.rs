use async_trait::async_trait;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};

use crate::{ConnectionAttributes, PacketProtVer, TcpContract};

pub struct MySbTcpSerializer {
    attr: ConnectionAttributes,
}

impl MySbTcpSerializer {
    pub fn new(attr: ConnectionAttributes) -> Self {
        Self { attr }
    }

    pub fn get_messages_to_deliver_packet_version(&self) -> PacketProtVer {
        self.attr.get(crate::tcp_message_id::NEW_MESSAGES)
    }
}

#[async_trait]
impl TcpSocketSerializer<TcpContract> for MySbTcpSerializer {
    const PING_PACKET_IS_SINGLETONE: bool = true;

    fn serialize(&self, contract: TcpContract) -> Vec<u8> {
        contract.serialize(self.attr.protocol_version)
    }
    fn get_ping(&self) -> TcpContract {
        TcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let result = TcpContract::deserialize(socket_reader, &self.attr).await?;
        Ok(result)
    }

    fn apply_packet(&mut self, contract: &TcpContract) -> bool {
        match contract {
            TcpContract::Greeting {
                name: _,
                protocol_version,
            } => {
                self.attr.protocol_version = *protocol_version;
                true
            }
            TcpContract::PacketVersions { packet_versions } => {
                self.attr.versions.update(packet_versions);
                true
            }
            _ => false,
        }
    }

    fn serialize_ref(&self, contract: &TcpContract) -> Vec<u8> {
        let contract = contract.clone();
        contract.serialize(self.attr.protocol_version)
    }
}
