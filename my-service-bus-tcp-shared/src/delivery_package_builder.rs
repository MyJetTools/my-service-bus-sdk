use my_service_bus_abstractions::MyServiceBusMessage;

use crate::{tcp_message_id, tcp_serializers::*, PacketProtVer, TcpContract};

pub struct DeliverTcpPacketBuilder {
    payload: Vec<u8>,
    amount_offset: usize,
    version: PacketProtVer,
    amount: i32,
}

impl DeliverTcpPacketBuilder {
    pub fn new(topic_id: &str, queue_id: &str, subscriber_id: i64, version: PacketProtVer) -> Self {
        let mut payload = Vec::new();
        payload.push(tcp_message_id::NEW_MESSAGES);
        pascal_string::serialize(&mut payload, topic_id);
        pascal_string::serialize(&mut payload, queue_id);
        i64::serialize(&mut payload, subscriber_id);

        let amount_offset = payload.len();
        i32::serialize(&mut payload, 0);

        Self {
            payload,
            amount_offset,
            version,
            amount: 0,
        }
    }

    pub fn append_packet(&mut self, msg: &impl MyServiceBusMessage) {
        crate::tcp_serializers::messages_to_deliver::serialize(
            &mut self.payload,
            msg,
            &self.version,
        );

        self.amount += 1;
    }

    pub fn get_result(mut self) -> TcpContract {
        let size = self.amount.to_le_bytes();
        let dest = &mut self.payload[self.amount_offset..self.amount_offset + 4];
        dest.copy_from_slice(size.as_slice());
        TcpContract::Raw(self.payload)
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use my_service_bus_abstractions::MySbMessage;

    use super::*;
    use crate::{PacketProtVer, TcpContract};

    #[tokio::test]
    async fn test_basic_use_case_v2() {
        const PROTOCOL_VERSION: i32 = 2;

        let version = PacketProtVer {
            packet_version: 1,
            protocol_version: PROTOCOL_VERSION,
        };

        let mut headers = HashMap::new();
        headers.insert("1".to_string(), "1".to_string());
        headers.insert("2".to_string(), "2".to_string());

        let msg1 = MySbMessage {
            id: 1.into(),
            content: vec![1, 1, 1],
            headers: Some(headers),
            attempt_no: 1,
        };

        let msg2 = MySbMessage {
            id: 2.into(),
            content: vec![2, 2, 2],
            headers: None,
            attempt_no: 2,
        };

        let mut builder =
            DeliverTcpPacketBuilder::new("test_topic", "test_queue", 15, version.clone());

        builder.append_packet(&msg1);
        builder.append_packet(&msg2);

        let tcp_contract = builder.get_result();

        let result = convert_from_raw(tcp_contract, &version).await;

        if let TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            mut messages,
        } = result
        {
            assert_eq!("test_topic", topic_id);
            assert_eq!("test_queue", queue_id);
            assert_eq!(15, confirmation_id);
            assert_eq!(2, messages.len());

            let result_msg1 = messages.remove(0);

            assert_eq!(1, result_msg1.attempt_no);
            assert_eq!(msg1.content, result_msg1.content);
            assert_eq!(true, result_msg1.headers.is_none());

            let result_msg2 = messages.remove(0);

            assert_eq!(2, result_msg2.attempt_no);
            assert_eq!(msg2.content, result_msg2.content);
            assert_eq!(true, result_msg2.headers.is_none());
        } else {
            panic!("We should not be ere")
        }
    }

    #[tokio::test]
    async fn test_basic_use_case_v3() {
        const PROTOCOL_VERSION: i32 = 3;

        let version = PacketProtVer {
            packet_version: 1,
            protocol_version: PROTOCOL_VERSION,
        };

        let mut headers = HashMap::new();
        headers.insert("1".to_string(), "1".to_string());
        headers.insert("2".to_string(), "2".to_string());

        let msg1 = MySbMessage {
            id: 1.into(),
            content: vec![1, 1, 1],
            headers: Some(headers),
            attempt_no: 1,
        };

        let msg2 = MySbMessage {
            id: 2.into(),
            content: vec![2, 2, 2],
            headers: None,
            attempt_no: 2,
        };

        let mut builder =
            DeliverTcpPacketBuilder::new("test_topic", "test_queue", 15, version.clone());

        builder.append_packet(&msg1);
        builder.append_packet(&msg2);

        let tcp_contract = builder.get_result();

        let result = convert_from_raw(tcp_contract, &version).await;

        if let TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            mut messages,
        } = result
        {
            assert_eq!("test_topic", topic_id);
            assert_eq!("test_queue", queue_id);
            assert_eq!(15, confirmation_id);
            assert_eq!(2, messages.len());

            let result_msg1 = messages.remove(0);

            assert_eq!(1, result_msg1.attempt_no);
            assert_eq!(msg1.content, result_msg1.content);
            assert_eq!(2, result_msg1.headers.unwrap().len());

            let result_msg2 = messages.remove(0);

            assert_eq!(2, result_msg2.attempt_no);
            assert_eq!(msg2.content, result_msg2.content);
            assert_eq!(true, result_msg2.headers.is_none());
        } else {
            panic!("We should not be ere")
        }
    }
}
