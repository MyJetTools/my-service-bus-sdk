use std::io::{Cursor, Read};

use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

pub async fn deserialize(reader: &mut impl SocketReader) -> Result<String, ReadingTcpContractFail> {
    let size = reader.read_byte().await? as usize;

    let mut result: Vec<u8> = Vec::with_capacity(size);
    unsafe { result.set_len(size) }

    reader.read_buf(&mut result).await?;

    Ok(String::from_utf8(result)?)
}

pub fn read_from_mem<'s>(reader: &mut Cursor<&'s [u8]>) -> Result<String, ReadingTcpContractFail> {
    let size = super::byte::read_from_mem(reader)? as usize;

    let mut result: Vec<u8> = Vec::with_capacity(size);
    unsafe { result.set_len(size) }

    reader.read_exact(&mut result)?;

    Ok(String::from_utf8(result)?)
}
