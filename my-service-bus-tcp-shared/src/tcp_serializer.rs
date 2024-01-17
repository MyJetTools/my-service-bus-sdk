use async_trait::async_trait;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer, TcpWriteBuffer,
};

use crate::{MySbSerializerState, MySbTcpContract};

pub struct MySbTcpSerializer;

impl Default for MySbTcpSerializer {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl TcpSocketSerializer<MySbTcpContract, MySbSerializerState> for MySbTcpSerializer {
    fn serialize(
        &self,
        out: &mut impl TcpWriteBuffer,
        contract: &MySbTcpContract,
        state: &MySbSerializerState,
    ) {
        contract.serialize(out, state)
    }

    fn get_ping(&self) -> MySbTcpContract {
        MySbTcpContract::Ping
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
        state: &MySbSerializerState,
    ) -> Result<MySbTcpContract, ReadingTcpContractFail> {
        let result = MySbTcpContract::deserialize(socket_reader, state).await?;

        Ok(result)
    }
}
