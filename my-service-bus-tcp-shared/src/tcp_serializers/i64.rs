use std::io::*;

pub fn serialize(data: &mut Vec<u8>, v: i64) {
    data.extend(&v.to_le_bytes());
}

pub fn read_from_mem<'s>(reader: &mut Cursor<&[u8]>) -> Result<i64> {
    let mut result = [0u8; 8];
    reader.read_exact(&mut result)?;

    Ok(i64::from_le_bytes(result))
}
