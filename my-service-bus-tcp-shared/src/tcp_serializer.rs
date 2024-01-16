use async_trait::async_trait;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer, TcpWriteBuffer,
};

use crate::{MySbSerializerMetadata, MySbTcpContract};

pub struct MySbTcpSerializer;

impl Default for MySbTcpSerializer {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl TcpSocketSerializer<MySbTcpContract, MySbSerializerMetadata> for MySbTcpSerializer {
    fn serialize(
        &self,
        out: &mut impl TcpWriteBuffer,
        contract: &MySbTcpContract,
        metadata: &MySbSerializerMetadata,
    ) {
        contract.serialize(out, metadata)
    }

    fn get_ping(&self) -> MySbTcpContract {
        MySbTcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        metadata: &MySbSerializerMetadata,
    ) -> Result<MySbTcpContract, ReadingTcpContractFail> {
        let result = MySbTcpContract::deserialize(socket_reader, metadata).await?;

        Ok(result)
    }
}
