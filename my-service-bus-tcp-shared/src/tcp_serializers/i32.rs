pub fn serialize(data: &mut Vec<u8>, v: i32) {
    data.extend(&v.to_le_bytes());
}
