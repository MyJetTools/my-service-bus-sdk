use std::io::*;

pub fn serialize(data: &mut Vec<u8>, v: i32) {
    data.extend(&v.to_le_bytes());
}

pub fn read_from_mem<'s>(reader: &mut Cursor<&[u8]>) -> Result<i32> {
    let mut result = [0u8; 4];
    reader.read_exact(&mut result)?;
    Ok(i32::from_le_bytes(result))
}
