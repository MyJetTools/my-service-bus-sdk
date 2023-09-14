use std::collections::HashMap;

use crate::{tcp_message_id, PacketProtVer};

//Now it's harcoded to NewMessages - since we are using it only for NewMessages for now
pub async fn convert_from_raw(
    src: crate::TcpContract,
    version: &PacketProtVer,
) -> crate::TcpContract {
    if let crate::TcpContract::Raw(payload) = src {
        let mut socket_reader = my_tcp_sockets::socket_reader::SocketReaderInMem::new(payload);

        let mut attr = crate::ConnectionAttributes::new(version.protocol_version);
        let mut versions = HashMap::new();
        versions.insert(tcp_message_id::NEW_MESSAGES, 1);
        attr.versions.update(&versions);

        return crate::TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();
    }

    panic!("This function works only with Raw payload");
}
