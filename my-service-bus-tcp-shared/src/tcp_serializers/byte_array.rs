pub fn serialize(data: &mut Vec<u8>, v: &[u8]) {
    let array_len = v.len() as i32;
    super::i32::serialize(data, array_len);
    data.extend(v);
}
