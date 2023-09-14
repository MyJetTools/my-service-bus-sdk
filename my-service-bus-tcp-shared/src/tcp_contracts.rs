use my_service_bus_abstractions::{
    publisher::MessageToPublish, queue_with_intervals::QueueIndexRange, subscriber::TopicQueueType,
    MySbMessage,
};
use my_tcp_sockets::socket_reader::{ReadingTcpContractFail, SocketReader};

use crate::ConnectionAttributes;

use super::tcp_message_id::*;

use std::collections::HashMap;

pub type RequestId = i64;

pub type ConfirmationId = i64;

#[derive(Debug, Clone)]
pub enum TcpContract {
    Ping,
    Pong,
    Greeting {
        name: String,
        protocol_version: i32,
    },
    Publish {
        topic_id: String,
        request_id: RequestId,
        persist_immediately: bool,
        data_to_publish: Vec<MessageToPublish>,
    },
    PublishResponse {
        request_id: RequestId,
    },
    Subscribe {
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    },
    SubscribeResponse {
        topic_id: String,
        queue_id: String,
    },
    Raw(Vec<u8>),
    NewMessages {
        topic_id: String,
        queue_id: String,
        confirmation_id: i64,
        messages: Vec<MySbMessage>,
    },
    NewMessagesConfirmation {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },
    CreateTopicIfNotExists {
        topic_id: String,
    },
    IntermediaryConfirm {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange>,
    },
    PacketVersions {
        packet_versions: HashMap<u8, i32>,
    },
    Reject {
        message: String,
    },
    AllMessagesConfirmedAsFail {
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },

    ConfirmSomeMessagesAsOk {
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange>,
    },
}

impl TcpContract {
    pub async fn deserialize<TSocketReader: SocketReader>(
        socket_reader: &mut TSocketReader,
        attr: &ConnectionAttributes,
    ) -> Result<TcpContract, ReadingTcpContractFail> {
        let packet_no = socket_reader.read_byte().await?;

        let result = match packet_no {
            PING => Ok(TcpContract::Ping {}),
            PONG => Ok(TcpContract::Pong {}),
            GREETING => {
                let name =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let protocol_version = socket_reader.read_i32().await?;

                let result = TcpContract::Greeting {
                    name,
                    protocol_version,
                };
                Ok(result)
            }
            PUBLISH => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let request_id = socket_reader.read_i64().await?;

                let messages_count = socket_reader.read_i32().await? as usize;

                let mut data_to_publish: Vec<MessageToPublish> = Vec::with_capacity(messages_count);

                if attr.protocol_version < 3 {
                    for _ in 0..messages_count {
                        let content = socket_reader.read_byte_array().await?;
                        data_to_publish.push(MessageToPublish {
                            headers: None,
                            content,
                        });
                    }
                } else {
                    for _ in 0..messages_count {
                        let headers =
                            crate::tcp_serializers::message_headers::deserialize(socket_reader)
                                .await?;
                        let content = socket_reader.read_byte_array().await?;
                        data_to_publish.push(MessageToPublish { headers, content });
                    }
                }

                let result = TcpContract::Publish {
                    topic_id,
                    request_id,
                    data_to_publish,
                    persist_immediately: socket_reader.read_bool().await?,
                };
                Ok(result)
            }
            PUBLISH_RESPONSE => {
                let request_id = socket_reader.read_i64().await?;
                let result = TcpContract::PublishResponse { request_id };

                Ok(result)
            }
            SUBSCRIBE => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_type = socket_reader.read_byte().await?;

                let queue_type = TopicQueueType::from_u8(queue_type);

                let result = TcpContract::Subscribe {
                    topic_id,
                    queue_id,
                    queue_type,
                };

                Ok(result)
            }
            SUBSCRIBE_RESPONSE => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let result = TcpContract::SubscribeResponse { topic_id, queue_id };

                Ok(result)
            }

            NEW_MESSAGES => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let records_len = socket_reader.read_i32().await? as usize;

                let mut messages = Vec::with_capacity(records_len);
                let version = attr.get(packet_no);
                for _ in 0..records_len {
                    let msg = crate::tcp_serializers::messages_to_deliver::deserialize(
                        socket_reader,
                        &version,
                    )
                    .await?;
                    messages.push(msg);
                }

                let result = TcpContract::NewMessages {
                    topic_id,
                    queue_id,
                    confirmation_id,
                    messages,
                };

                Ok(result)
            }
            ALL_MESSAGES_DELIVERED_CONFIRMATION => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::NewMessagesConfirmation {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }
            CREATE_TOPIC_IF_NOT_EXISTS => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;

                let result = TcpContract::CreateTopicIfNotExists { topic_id };

                Ok(result)
            }

            REJECT => {
                let message =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let result = TcpContract::Reject { message };
                Ok(result)
            }

            PACKET_VERSIONS => {
                let len = socket_reader.read_byte().await?;

                let mut packet_versions: HashMap<u8, i32> = HashMap::new();

                for _ in 0..len {
                    let p = socket_reader.read_byte().await?;
                    let v = socket_reader.read_i32().await?;
                    packet_versions.insert(p, v);
                }

                let result = TcpContract::PacketVersions { packet_versions };

                Ok(result)
            }

            ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = TcpContract::AllMessagesConfirmedAsFail {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }

            CONFIRM_SOME_MESSAGES_AS_OK => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let delivered =
                    crate::tcp_serializers::queue_with_intervals::deserialize(socket_reader)
                        .await?;

                let result = TcpContract::ConfirmSomeMessagesAsOk {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            INTERMEDIARY_CONFIRM => {
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let delivered =
                    crate::tcp_serializers::queue_with_intervals::deserialize(socket_reader)
                        .await?;

                let result = TcpContract::IntermediaryConfirm {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            _ => Err(ReadingTcpContractFail::InvalidPacketId(packet_no)),
        };

        return result;
    }

    pub fn serialize(self, protocol_version: i32) -> Vec<u8> {
        match self {
            TcpContract::Ping {} => {
                let mut result: Vec<u8> = Vec::with_capacity(1);
                result.push(PING);
                result
            }
            TcpContract::Pong {} => {
                let mut result: Vec<u8> = Vec::with_capacity(1);
                result.push(PONG);
                result
            }
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(GREETING);
                crate::tcp_serializers::pascal_string::serialize(&mut result, name.as_str());
                crate::tcp_serializers::i32::serialize(&mut result, protocol_version);
                result
            }
            TcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => Self::compile_publish_payload(
                topic_id.as_str(),
                request_id,
                data_to_publish.as_slice(),
                persist_immediately,
                protocol_version,
            ),

            TcpContract::PublishResponse { request_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(PUBLISH_RESPONSE);
                crate::tcp_serializers::i64::serialize(&mut result, request_id);
                result
            }
            TcpContract::Subscribe {
                topic_id,
                queue_id,
                queue_type,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(SUBSCRIBE);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                crate::tcp_serializers::byte::serialize(&mut result, queue_type.into_u8());
                result
            }
            TcpContract::SubscribeResponse { topic_id, queue_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(SUBSCRIBE_RESPONSE);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                result
            }
            TcpContract::Raw(payload) => payload,
            TcpContract::NewMessages {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                messages: _,
            } => {
                panic!(
                    "This packet is not used by server. Server uses optimized version of the packet"
                );
            }
            TcpContract::NewMessagesConfirmation {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(ALL_MESSAGES_DELIVERED_CONFIRMATION);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                crate::tcp_serializers::i64::serialize(&mut result, confirmation_id);
                result
            }
            TcpContract::CreateTopicIfNotExists { topic_id } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(CREATE_TOPIC_IF_NOT_EXISTS);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                result
            }
            TcpContract::IntermediaryConfirm {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(INTERMEDIARY_CONFIRM);
                result.push(packet_version);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                crate::tcp_serializers::i64::serialize(&mut result, confirmation_id);

                crate::tcp_serializers::queue_with_intervals::serialize(&mut result, &delivered);
                result
            }
            TcpContract::PacketVersions { packet_versions } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(PACKET_VERSIONS);

                let data_len = packet_versions.len() as u8;
                crate::tcp_serializers::byte::serialize(&mut result, data_len);

                for kv in packet_versions {
                    crate::tcp_serializers::byte::serialize(&mut result, kv.0);
                    crate::tcp_serializers::i32::serialize(&mut result, kv.1);
                }
                result
            }
            TcpContract::Reject { message } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(REJECT);
                crate::tcp_serializers::pascal_string::serialize(&mut result, message.as_str());
                result
            }
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                crate::tcp_serializers::i64::serialize(&mut result, confirmation_id);
                result
            }

            TcpContract::ConfirmSomeMessagesAsOk {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                let mut result: Vec<u8> = Vec::new();
                result.push(CONFIRM_SOME_MESSAGES_AS_OK);
                result.push(packet_version);
                crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                crate::tcp_serializers::i64::serialize(&mut result, confirmation_id);
                crate::tcp_serializers::queue_with_intervals::serialize(&mut result, &delivered);
                result
            }
        }
    }

    pub fn compile_publish_payload(
        topic_id: &str,
        request_id: i64,
        data_to_publish: &[MessageToPublish],
        persist_immediately: bool,
        protocol_version: i32,
    ) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        result.push(PUBLISH);
        crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id);
        crate::tcp_serializers::i64::serialize(&mut result, request_id);
        crate::tcp_serializers::messages_to_publish::serialize(
            &mut result,
            &data_to_publish,
            protocol_version,
        );

        crate::tcp_serializers::bool::serialize(&mut result, persist_immediately);
        result
    }
}

impl my_tcp_sockets::tcp_connection::TcpContract for TcpContract {
    fn is_pong(&self) -> bool {
        if let TcpContract::Pong = self {
            return true;
        }

        false
    }
}
#[cfg(test)]
mod tests {

    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    use super::*;

    #[tokio::test]
    async fn test_ping_packet() {
        let tcp_packet = TcpContract::Ping;

        let serialized_data: Vec<u8> = tcp_packet.serialize(2);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);
        let attr = ConnectionAttributes::new(0);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Ping => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_pong_packet() {
        let tcp_packet = TcpContract::Pong;

        let serialized_data: Vec<u8> = tcp_packet.serialize(2);
        let mut socket_reader = SocketReaderInMem::new(serialized_data);
        let attr = ConnectionAttributes::new(0);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Pong => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_greeting_packet() {
        let test_app_name = "test_app";
        let test_protocol_version = 2;

        let tcp_packet = TcpContract::Greeting {
            name: test_app_name.to_string(),
            protocol_version: test_protocol_version,
        };
        let serialized_data: Vec<u8> = tcp_packet.serialize(0);
        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let attr = ConnectionAttributes::new(0);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Greeting {
                name,
                protocol_version,
            } => {
                assert_eq!(test_app_name, name);
                assert_eq!(test_protocol_version, protocol_version);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_packet_v2() {
        const PROTOCOL_VERSION: i32 = 2;

        let request_id_test = 1;

        let message_to_publish = MessageToPublish {
            content: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
            headers: None,
        };

        let data_test = vec![message_to_publish];
        let topic_test = String::from("test-topic");
        let persist_test = true;

        let tcp_packet = TcpContract::Publish {
            data_to_publish: data_test,
            persist_immediately: persist_test,
            request_id: request_id_test,
            topic_id: topic_test,
        };
        let attr = ConnectionAttributes::new(PROTOCOL_VERSION);
        let serialized_data: Vec<u8> = tcp_packet.serialize(attr.protocol_version);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Publish {
                data_to_publish,
                persist_immediately,
                request_id,
                topic_id,
            } => {
                assert_eq!(request_id_test, request_id);
                assert_eq!(String::from("test-topic"), topic_id);
                assert_eq!(persist_test, persist_immediately);

                let data_test = vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]];

                for index in 0..data_to_publish[0].content.len() {
                    assert_eq!(data_test[0][index], data_to_publish[0].content[index]);
                }
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_packet_v3() {
        const PROTOCOL_VERSION: i32 = 3;

        let request_id_test = 1;

        let mut headers = HashMap::new();
        headers.insert("key1".to_string(), "value1".to_string());
        headers.insert("key2".to_string(), "value2".to_string());

        let message_to_publish = MessageToPublish {
            content: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
            headers: Some(headers),
        };

        let data_test = vec![message_to_publish];
        let topic_test = String::from("test-topic");
        let persist_test = true;

        let tcp_packet = TcpContract::Publish {
            data_to_publish: data_test,
            persist_immediately: persist_test,
            request_id: request_id_test,
            topic_id: topic_test,
        };

        let attr = ConnectionAttributes::new(PROTOCOL_VERSION);
        let serialized_data: Vec<u8> = tcp_packet.serialize(attr.protocol_version);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Publish {
                mut data_to_publish,
                persist_immediately,
                request_id,
                topic_id,
            } => {
                assert_eq!(request_id_test, request_id);
                assert_eq!(String::from("test-topic"), topic_id);
                assert_eq!(persist_test, persist_immediately);

                let data_test = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0];

                assert_eq!(1, data_to_publish.len());

                let el0 = data_to_publish.remove(0);

                let mut headers = el0.headers.unwrap();

                assert_eq!(data_test, el0.content);
                assert_eq!(2, headers.len());

                assert_eq!("value1", headers.remove("key1").unwrap());
                assert_eq!("value2", headers.remove("key2").unwrap());
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_publish_response_packet() {
        const PROTOCOL_VERSION: i32 = 2;

        let request_id_test = 1;

        let tcp_packet = TcpContract::PublishResponse {
            request_id: request_id_test,
        };

        let mut attr = ConnectionAttributes::new(PROTOCOL_VERSION);
        attr.protocol_version = PROTOCOL_VERSION;

        let serialized_data: Vec<u8> = tcp_packet.serialize(PROTOCOL_VERSION);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::PublishResponse { request_id } => {
                assert_eq!(request_id_test, request_id);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_subscribe_packet() {
        const PROTOCOL_VERSION: i32 = 2;
        let queue_id_test = String::from("queue");
        let topic_id_test = String::from("topic");
        let queue_type_test = TopicQueueType::PermanentWithSingleConnection;

        let tcp_packet = TcpContract::Subscribe {
            queue_id: queue_id_test,
            topic_id: topic_id_test,
            queue_type: queue_type_test,
        };

        let attr = ConnectionAttributes::new(PROTOCOL_VERSION);
        let serialized_data: Vec<u8> = tcp_packet.serialize(PROTOCOL_VERSION);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = TcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            TcpContract::Subscribe {
                queue_id,
                queue_type,
                topic_id,
            } => {
                let queue_id_test = String::from("queue");
                let topic_id_test = String::from("topic");

                assert_eq!(queue_id_test, queue_id);
                assert_eq!(topic_id_test, topic_id);
                match queue_type {
                    TopicQueueType::PermanentWithSingleConnection => {}
                    _ => {
                        panic!("Invalid Queue Type");
                    }
                };
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }
}
