use crate::TcpProtocolVersion;
use my_service_bus_abstractions::publisher::MessageToPublish;
use my_tcp_sockets::TcpWriteBuffer;

pub fn serialize(
    write_buffer: &mut impl TcpWriteBuffer,
    v: &[MessageToPublish],
    protocol_version: TcpProtocolVersion,
) {
    if protocol_version.get_value() < 3 {
        serialize_v2(write_buffer, v)
    } else {
        serialize_v3(write_buffer, v)
    }
}

pub fn serialize_v2(write_buffer: &mut impl TcpWriteBuffer, v: &[MessageToPublish]) {
    let array_len = v.len() as i32;
    write_buffer.write_i32(array_len);
    //super::i32::serialize(data, array_len);

    for msg in v {
        write_buffer.write_byte_array(&msg.content);
        //super::byte_array::serialize(data, &arr.content);
    }
}

pub fn serialize_v3(write_buffer: &mut impl TcpWriteBuffer, v: &[MessageToPublish]) {
    let array_len = v.len() as i32;
    write_buffer.write_i32(array_len);
    //super::i32::serialize(data, array_len);

    for item in v {
        super::message_headers::serialize(write_buffer, &item.headers);
        write_buffer.write_byte_array(&item.content);
        //super::byte_array::serialize(data, &item.content);
    }
}
