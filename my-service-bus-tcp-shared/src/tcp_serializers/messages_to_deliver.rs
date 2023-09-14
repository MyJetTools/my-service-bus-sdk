use my_service_bus_abstractions::{MySbMessage, MyServiceBusMessage};

use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

use crate::PacketProtVer;

pub fn serialize(dest: &mut Vec<u8>, msg: &impl MyServiceBusMessage, version: &PacketProtVer) {
    if version.protocol_version < 3 {
        serialize_v2(dest, msg, version.packet_version);
    } else {
        serialize_v3(dest, msg);
    }
}

pub fn serialize_v2(dest: &mut Vec<u8>, msg: &impl MyServiceBusMessage, packet_version: i32) {
    crate::tcp_serializers::i64::serialize(dest, msg.get_id().get_value());

    if packet_version == 1 {
        crate::tcp_serializers::i32::serialize(dest, msg.get_attempt_no());
    }
    super::byte_array::serialize(dest, msg.get_content());
}

pub fn serialize_v3(dest: &mut Vec<u8>, msg: &impl MyServiceBusMessage) {
    crate::tcp_serializers::i64::serialize(dest, msg.get_id().get_value());
    crate::tcp_serializers::i32::serialize(dest, msg.get_attempt_no());
    super::message_headers::serialize(dest, msg.get_headers());
    super::byte_array::serialize(dest, msg.get_content());
}

pub async fn deserialize<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
    version: &PacketProtVer,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    if version.protocol_version < 3 {
        return deserialize_v2(socket_reader, version.packet_version).await;
    }

    return deserialize_v3(socket_reader).await;
}

pub async fn deserialize_v2<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
    packet_version: i32,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    let id = socket_reader.read_i64().await?;

    let attempt_no = if packet_version == 1 {
        socket_reader.read_i32().await?
    } else {
        0
    };

    let content = socket_reader.read_byte_array().await?;

    let result = MySbMessage {
        id: id.into(),
        headers: None,
        attempt_no,
        content,
    };

    Ok(result)
}

pub async fn deserialize_v3<TSocketReader: SocketReader>(
    socket_reader: &mut TSocketReader,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    let id = socket_reader.read_i64().await?;

    let attempt_no = socket_reader.read_i32().await?;

    let headers = crate::tcp_serializers::message_headers::deserialize(socket_reader).await?;

    let content = socket_reader.read_byte_array().await?;

    let result = MySbMessage {
        id: id.into(),
        headers,
        attempt_no,
        content,
    };

    Ok(result)
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use my_service_bus_abstractions::MySbMessage;
    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    use crate::PacketProtVer;

    #[tokio::test]
    pub async fn test_v2() {
        let version = PacketProtVer {
            protocol_version: 2,
            packet_version: 1,
        };

        let mut headers = HashMap::new();
        headers.insert("key1".to_string(), "value1".to_string());

        let src_msg = MySbMessage {
            id: 1.into(),
            content: vec![0u8, 1u8, 2u8],
            headers: Some(headers),
            attempt_no: 1,
        };

        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, &src_msg, &version);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader, &version)
            .await
            .unwrap();

        assert_eq!(src_msg.id, result.id);
        assert_eq!(src_msg.content, result.content);
        assert_eq!(true, result.headers.is_none());
    }

    #[tokio::test]
    pub async fn test_v3() {
        let version = PacketProtVer {
            protocol_version: 3,
            packet_version: 1,
        };

        let mut headers = HashMap::new();
        headers.insert("key1".to_string(), "value1".to_string());

        let src_msg = MySbMessage {
            id: 1.into(),
            content: vec![0u8, 1u8, 2u8],
            headers: Some(headers),
            attempt_no: 1,
        };

        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, &src_msg, &version);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader, &version)
            .await
            .unwrap();

        assert_eq!(src_msg.id, result.id);
        assert_eq!(src_msg.content, result.content);

        let headers = result.headers.unwrap();
        assert_eq!(1, headers.len());

        assert_eq!("value1", headers.get("key1").unwrap());
    }
}
