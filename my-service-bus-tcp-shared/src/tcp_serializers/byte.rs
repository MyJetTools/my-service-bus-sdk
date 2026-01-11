use std::io::*;

pub fn serialize(data: &mut Vec<u8>, v: u8) {
    data.push(v);
}

pub fn read_from_mem<'s>(reader: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut result = [0u8; 1];
    reader.read_exact(&mut result)?;

    Ok(result[0])
}
