pub fn serialize(data: &mut Vec<u8>, v: &Vec<Vec<u8>>) {
    let array_len = v.len() as i32;
    super::i32::serialize(data, array_len);

    for arr in v {
        super::byte_array::serialize(data, arr);
    }
}
