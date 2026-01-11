use std::io::*;

pub fn serialize(data: &mut Vec<u8>, v: &[u8]) {
    let array_len = v.len() as i32;
    super::i32::serialize(data, array_len);
    data.extend(v);
}

pub fn read_from_mem(reader: &mut Cursor<&[u8]>) -> Result<Vec<u8>> {
    let size = super::i32::read_from_mem(reader)? as usize;

    let mut result = Vec::with_capacity(size);

    unsafe {
        result.set_len(size);
    }

    reader.read_exact(&mut result)?;

    Ok(result)
}
