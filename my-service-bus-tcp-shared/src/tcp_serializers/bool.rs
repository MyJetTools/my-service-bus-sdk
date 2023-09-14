pub fn serialize(data: &mut Vec<u8>, value: bool) {
    if value {
        data.push(1);
    } else {
        data.push(0);
    }
}
