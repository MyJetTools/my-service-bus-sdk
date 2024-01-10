use crate::PacketProtVer;

pub fn serialize(payload: &mut Vec<u8>, value: i64, ver: &PacketProtVer) {
    if ver.tcp_protocol_version.get_value() < 2 {
        super::i32::serialize(payload, value as i32);
    } else {
        super::i64::serialize(payload, value);
    }
}
