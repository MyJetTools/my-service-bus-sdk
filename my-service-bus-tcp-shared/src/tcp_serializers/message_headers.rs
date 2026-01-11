use std::io::Cursor;

use my_service_bus_abstractions::SbMessageHeaders;
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpWriteBuffer,
};

pub async fn deserialize<TSocketReader: SocketReader>(
    reader: &mut TSocketReader,
) -> Result<SbMessageHeaders, ReadingTcpContractFail> {
    let headers_count = reader.read_byte().await? as usize;

    let mut result = SbMessageHeaders::with_capacity(headers_count);

    for _ in 0..headers_count {
        let key = super::pascal_string::deserialize(reader).await?;
        let value = super::pascal_string::deserialize(reader).await?;

        result = result.add(key, value);
    }

    Ok(result)
}

pub fn read_from_mem(
    reader: &mut Cursor<&[u8]>,
) -> Result<SbMessageHeaders, ReadingTcpContractFail> {
    let headers_count = super::byte::read_from_mem(reader)? as usize;

    let mut result = SbMessageHeaders::with_capacity(headers_count);

    for _ in 0..headers_count {
        let key = super::pascal_string::read_from_mem(reader)?;
        let value = super::pascal_string::read_from_mem(reader)?;

        result = result.add(key, value);
    }

    Ok(result)
}

pub fn serialize(write_buffer: &mut impl TcpWriteBuffer, headers: &SbMessageHeaders) {
    let mut headers_count = headers.len();

    if headers_count > 255 {
        headers_count = 255;
    }

    write_buffer.write_byte(headers_count as u8);

    let mut i = 0;

    for (key, value) in headers.iter() {
        if i == 255 {
            break;
        }

        write_buffer.write_pascal_string(key);
        //super::pascal_string::serialize(data, key);

        write_buffer.write_pascal_string(value);
        //super::pascal_string::serialize(data, value);

        i += 1;
    }

    /*
    match headers {
        Some(headers) => {

        }
        None => {
            write_buffer.write_byte(0);
        }
    }
     */
}

#[cfg(test)]
mod test {

    use my_service_bus_abstractions::SbMessageHeaders;
    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    #[tokio::test]
    pub async fn test_headers() {
        let headers = SbMessageHeaders::new()
            .add("Key1", "Value1")
            .add("Key2", "Value2");
        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, &headers);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader).await.unwrap();

        assert_eq!(2, result.len());
    }

    #[tokio::test]
    pub async fn test_empty_headers() {
        let headers = SbMessageHeaders::new();
        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, &headers);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader).await.unwrap();
        assert_eq!(0, result.len());
    }
}
