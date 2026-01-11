use std::io::Cursor;

use my_service_bus_abstractions::{MySbMessage, MyServiceBusMessage, SbMessageHeaders};

use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

use crate::PacketProtVer;

pub const MESSAGE_LAST_VERSION: u8 = 3;

pub fn serialize(dest: &mut Vec<u8>, msg: &impl MyServiceBusMessage, version: &PacketProtVer) {
    if version.tcp_protocol_version.get_value() < 3 {
        serialize_v2(dest, msg, version.packet_version);
    } else {
        serialize_v3(dest, msg);
    }
}

pub fn serialize_v2(dest: &mut Vec<u8>, msg: &impl MyServiceBusMessage, packet_version: u8) {
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

pub async fn deserialize<TSocketReader: SocketReader + Send + Sync + 'static>(
    socket_reader: &mut TSocketReader,
    version: &PacketProtVer,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    if version.tcp_protocol_version.get_value() < 3 {
        return deserialize_v2(socket_reader, version.packet_version).await;
    }

    return deserialize_v3(socket_reader).await;
}

pub async fn deserialize_v2<TSocketReader: SocketReader + Send + Sync + 'static>(
    socket_reader: &mut TSocketReader,
    packet_version: u8,
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
        headers: SbMessageHeaders::new(),
        attempt_no,
        content,
    };

    Ok(result)
}

pub async fn deserialize_v3<TSocketReader: SocketReader + Send + Sync + 'static>(
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

pub fn read_from_mem(
    reader: &mut Cursor<&[u8]>,
    version: &PacketProtVer,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    if version.tcp_protocol_version.get_value() < 3 {
        return read_from_mem_v2(reader, version.packet_version);
    }

    return read_from_mem_v3(reader);
}

pub fn read_from_mem_v2(
    reader: &mut Cursor<&[u8]>,
    packet_version: u8,
) -> Result<MySbMessage, ReadingTcpContractFail> {
    let id = super::i64::read_from_mem(reader)?;

    let attempt_no = if packet_version == 1 {
        super::i32::read_from_mem(reader)?
    } else {
        0
    };

    let content = super::byte_array::read_from_mem(reader)?;

    let result = MySbMessage {
        id: id.into(),
        headers: SbMessageHeaders::new(),
        attempt_no,
        content,
    };

    Ok(result)
}

pub fn read_from_mem_v3(reader: &mut Cursor<&[u8]>) -> Result<MySbMessage, ReadingTcpContractFail> {
    let id = super::i64::read_from_mem(reader)?;

    let attempt_no = super::i32::read_from_mem(reader)?;

    let headers = crate::tcp_serializers::message_headers::read_from_mem(reader)?;

    let content = super::byte_array::read_from_mem(reader)?;

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

    use my_service_bus_abstractions::{MySbMessage, SbMessageHeaders};
    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    use crate::PacketProtVer;

    #[tokio::test]
    pub async fn test_v2() {
        let version = PacketProtVer {
            tcp_protocol_version: 2i32.into(),
            packet_version: 1,
        };

        let headers = SbMessageHeaders::new().add("key1", "value1");

        let src_msg = MySbMessage {
            id: 1.into(),
            content: vec![0u8, 1u8, 2u8],
            headers,
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
        assert_eq!(0, result.headers.len());
    }

    #[tokio::test]
    pub async fn test_v3() {
        let version = PacketProtVer {
            tcp_protocol_version: 3i32.into(),
            packet_version: 1,
        };

        let headers = SbMessageHeaders::new().add("key1", "value1");

        let src_msg = MySbMessage {
            id: 1.into(),
            content: vec![0u8, 1u8, 2u8],
            headers,
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

        assert_eq!(1, result.headers.len());
        assert_eq!("value1", result.headers.get("key1").unwrap());
    }
}
