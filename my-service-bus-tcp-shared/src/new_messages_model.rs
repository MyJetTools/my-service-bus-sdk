use std::io::Cursor;

use my_service_bus_abstractions::MySbMessage;

use crate::PacketProtVer;

#[derive(Debug, Clone)]
pub struct NewMessagesModel {
    pub topic_id: String,
    pub queue_id: String,
    pub confirmation_id: i64,
    pub messages: Vec<MySbMessage>,
}

impl NewMessagesModel {
    pub fn deserialize(
        src: &[u8],
        packet_version: &PacketProtVer,
    ) -> Result<Self, my_tcp_sockets::socket_reader::ReadingTcpContractFail> {
        let mut cursor = Cursor::new(src);

        let packet = super::tcp_serializers::byte::read_from_mem(&mut cursor)?;

        if packet != crate::tcp_message_id::NEW_MESSAGES {
            return Err(
                my_tcp_sockets::socket_reader::ReadingTcpContractFail::InvalidPacketId(packet),
            );
        }

        let topic_id = super::tcp_serializers::pascal_string::read_from_mem(&mut cursor)?;

        let queue_id = super::tcp_serializers::pascal_string::read_from_mem(&mut cursor)?;

        let confirmation_id = super::tcp_serializers::i64::read_from_mem(&mut cursor)?;

        let records_amount = super::tcp_serializers::i32::read_from_mem(&mut cursor)? as usize;

        let mut messages = Vec::with_capacity(records_amount);

        for _ in 0..records_amount {
            let msg = crate::tcp_serializers::messages_to_deliver::read_from_mem(
                &mut cursor,
                &packet_version,
            )?;
            messages.push(msg);
        }

        let result = Self {
            topic_id,
            queue_id,
            confirmation_id,
            messages,
        };

        Ok(result)
    }
}
