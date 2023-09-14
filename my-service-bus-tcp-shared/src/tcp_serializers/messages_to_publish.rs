use my_service_bus_abstractions::publisher::MessageToPublish;

pub fn serialize(data: &mut Vec<u8>, v: &[MessageToPublish], protocol_version: i32) {
    if protocol_version < 3 {
        serialize_v2(data, v)
    } else {
        serialize_v3(data, v)
    }
}

pub fn serialize_v2(data: &mut Vec<u8>, v: &[MessageToPublish]) {
    let array_len = v.len() as i32;
    super::i32::serialize(data, array_len);

    for arr in v {
        super::byte_array::serialize(data, &arr.content);
    }
}

pub fn serialize_v3(data: &mut Vec<u8>, v: &[MessageToPublish]) {
    let array_len = v.len() as i32;
    super::i32::serialize(data, array_len);

    for item in v {
        super::message_headers::serialize(data, item.headers.as_ref());
        super::byte_array::serialize(data, &item.content);
    }
}
