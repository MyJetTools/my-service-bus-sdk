use std::collections::HashMap;

use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

pub async fn deserialize<TSocketReader: SocketReader>(
    reader: &mut TSocketReader,
) -> Result<Option<HashMap<String, String>>, ReadingTcpContractFail> {
    let headers_count = reader.read_byte().await? as usize;

    if headers_count == 0 {
        return Ok(None);
    }

    let mut result = HashMap::with_capacity(headers_count);

    for _ in 0..headers_count {
        let key = super::pascal_string::deserialize(reader).await?;
        let value = super::pascal_string::deserialize(reader).await?;

        result.insert(key, value);
    }

    Ok(Some(result))
}

pub fn serialize(data: &mut Vec<u8>, headers: Option<&HashMap<String, String>>) {
    match headers {
        Some(headers) => {
            let mut headers_count = headers.len();

            if headers_count > 255 {
                headers_count = 255;
            }

            data.push(headers_count as u8);

            let mut i = 0;

            for (key, value) in headers {
                if i == 255 {
                    break;
                }

                super::pascal_string::serialize(data, key);
                super::pascal_string::serialize(data, value);

                i += 1;
            }
        }
        None => {
            data.push(0);
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    #[tokio::test]
    pub async fn test_headers() {
        let mut headers = HashMap::new();
        headers.insert("Key1".to_string(), "Value1".to_string());
        headers.insert("Key2".to_string(), "Value2".to_string());
        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, Some(&headers));

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader).await.unwrap();

        let result = result.unwrap();
        assert_eq!(2, result.len());
    }

    #[tokio::test]
    pub async fn test_empty_headers() {
        let headers = HashMap::new();
        let mut serialized_data = Vec::new();

        super::serialize(&mut serialized_data, Some(&headers));

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = super::deserialize(&mut socket_reader).await.unwrap();
        assert_eq!(true, result.is_none());
    }
}
