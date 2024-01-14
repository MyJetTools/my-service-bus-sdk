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
impl TcpSocketSerializer<TcpContract, ()> for MySbTcpSerializer {
    fn serialize(&self, out: &mut impl TcpWriteBuffer, contract: &TcpContract, metadata: &()) {
        //contract.serialize(out, metadata)
    }

    fn get_ping(&self) -> TcpContract {
        TcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        metadata: &(),
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let result = TcpContract::deserialize(socket_reader).await?;

        Ok(result)
    }
}
