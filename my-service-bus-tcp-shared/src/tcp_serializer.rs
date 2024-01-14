use async_trait::async_trait;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer, TcpWriteBuffer,
};

use crate::{MySbSerializerMetadata, TcpContract};

pub struct MySbTcpSerializer;

impl Default for MySbTcpSerializer {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl TcpSocketSerializer<TcpContract, MySbSerializerMetadata> for MySbTcpSerializer {
    fn serialize(
        &self,
        out: &mut impl TcpWriteBuffer,
        contract: &TcpContract,
        metadata: &MySbSerializerMetadata,
    ) {
        contract.serialize(out, metadata)
    }

    fn get_ping(&self) -> TcpContract {
        TcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        metadata: Option<&MySbSerializerMetadata>,
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let result = TcpContract::deserialize(socket_reader, metadata).await?;

        Ok(result)
    }

    /*
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
     */
}
