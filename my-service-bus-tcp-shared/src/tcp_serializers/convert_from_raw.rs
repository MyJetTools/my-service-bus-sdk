use crate::MySbSerializerMetadata;

//Now it's harcoded to NewMessages - since we are using it only for NewMessages for now
pub async fn convert_from_raw(
    src: crate::TcpContract,
    metadata: Option<&MySbSerializerMetadata>,
) -> crate::TcpContract {
    if let crate::TcpContract::Raw(payload) = src {
        let mut socket_reader = my_tcp_sockets::socket_reader::SocketReaderInMem::new(payload);

        return crate::TcpContract::deserialize(&mut socket_reader, metadata)
            .await
            .unwrap();
    }

    panic!("This function works only with Raw payload");
}
