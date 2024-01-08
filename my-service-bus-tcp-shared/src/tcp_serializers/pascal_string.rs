use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

pub async fn deserialize<TSocketString: SocketReader>(
    reader: &mut TSocketString,
) -> Result<String, ReadingTcpContractFail> {
    let size = reader.read_byte().await? as usize;

    let mut result: Vec<u8> = Vec::with_capacity(size);
    unsafe { result.set_len(size) }

    reader.read_buf(&mut result).await?;

    Ok(String::from_utf8(result)?)
}
