use my_service_bus_abstractions::{
    publisher::MessageToPublish, queue_with_intervals::QueueIndexRange, subscriber::TopicQueueType,
    SbMessageHeaders,
};
use my_tcp_sockets::{
    socket_reader::{ReadingTcpContractFail, SocketReader},
    TcpWriteBuffer,
};

use crate::{MySbSerializerState, NewMessagesModel};

use super::tcp_message_id::*;

use std::collections::HashMap;

pub type RequestId = i64;

pub type ConfirmationId = i64;

/// Identifier a node assigns to each virtual subscriber it registers with the
/// master on behalf of a downstream client. Unique within a node↔master TCP
/// session. The master treats each virtual subscriber as a separate logical
/// subscriber for queue load-balancing purposes.
pub type VirtualSubscriberId = i64;

/// Sequence number used by the node to correlate `NodePublish` requests with
/// `NodePublishResponse` acks. Monotonically increasing within a node↔master
/// session.
pub type NodeSequenceNumber = i64;

/// One topic's worth of messages inside a `NodePublish` packet.
#[derive(Debug, Clone)]
pub struct NodeTopicPublish {
    pub topic_id: String,
    pub persist_immediately: bool,
    pub data_to_publish: Vec<MessageToPublish>,
}

/// Ack info for one topic inside a `NodePublishResponse`. The range
/// `[first_message_id, last_message_id]` is the contiguous set of IDs the master
/// assigned to the messages in that topic for this packet.
#[derive(Debug, Clone)]
pub struct NodeTopicAck {
    pub topic_id: String,
    pub first_message_id: i64,
    pub last_message_id: i64,
}

#[derive(Debug, Clone)]
pub enum MySbTcpContract {
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
    NewMessages(NewMessagesModel),
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
        delivered: Vec<QueueIndexRange<i64>>,
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
        delivered: Vec<QueueIndexRange<i64>>,
    },

    // ----- Node-mode packets (see my-service-bus-node) -----
    NodeGreeting {
        name: String,
        protocol_version: i32,
    },
    NodePublish {
        sequence_number: NodeSequenceNumber,
        topics: Vec<NodeTopicPublish>,
    },
    NodePublishResponse {
        sequence_number: NodeSequenceNumber,
        topics: Vec<NodeTopicAck>,
    },
    NodeSubscribe {
        virtual_subscriber_id: VirtualSubscriberId,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    },
    NodeSubscribeResponse {
        virtual_subscriber_id: VirtualSubscriberId,
        topic_id: String,
        queue_id: String,
    },
    NodeUnsubscribe {
        virtual_subscriber_id: VirtualSubscriberId,
    },
    NodeNewMessages {
        virtual_subscriber_id: VirtualSubscriberId,
        new_messages: NewMessagesModel,
    },
    NodeNewMessagesConfirmation {
        virtual_subscriber_id: VirtualSubscriberId,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },
    NodeAllMessagesConfirmedAsFail {
        virtual_subscriber_id: VirtualSubscriberId,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
    },
    NodeConfirmSomeMessagesAsOk {
        virtual_subscriber_id: VirtualSubscriberId,
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange<i64>>,
    },
    NodeIntermediaryConfirm {
        virtual_subscriber_id: VirtualSubscriberId,
        packet_version: u8,
        topic_id: String,
        queue_id: String,
        confirmation_id: ConfirmationId,
        delivered: Vec<QueueIndexRange<i64>>,
    },
}

impl MySbTcpContract {
    pub async fn deserialize<TSocketReader: SocketReader + Send + Sync + 'static>(
        socket_reader: &mut TSocketReader,
        serializer_metadata: &MySbSerializerState,
    ) -> Result<MySbTcpContract, ReadingTcpContractFail> {
        let packet_no = socket_reader.read_byte().await?;

        let result = match packet_no {
            PING => Ok(MySbTcpContract::Ping {}),
            PONG => Ok(MySbTcpContract::Pong {}),
            GREETING => {
                let name =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let protocol_version = socket_reader.read_i32().await?;

                let result = MySbTcpContract::Greeting {
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

                if serializer_metadata.tcp_protocol_version.get_value() < 3 {
                    for _ in 0..messages_count {
                        let content = socket_reader.read_byte_array().await?;
                        data_to_publish.push(MessageToPublish {
                            headers: SbMessageHeaders::new(),
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

                let result = MySbTcpContract::Publish {
                    topic_id,
                    request_id,
                    data_to_publish,
                    persist_immediately: socket_reader.read_bool().await?,
                };
                Ok(result)
            }
            PUBLISH_RESPONSE => {
                let request_id = socket_reader.read_i64().await?;
                let result = MySbTcpContract::PublishResponse { request_id };

                Ok(result)
            }
            SUBSCRIBE => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_type = socket_reader.read_byte().await?;

                let queue_type = TopicQueueType::from_u8(queue_type);

                let result = MySbTcpContract::Subscribe {
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
                let result = MySbTcpContract::SubscribeResponse { topic_id, queue_id };

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
                let version = serializer_metadata.get(packet_no);

                for _ in 0..records_len {
                    let msg = crate::tcp_serializers::messages_to_deliver::deserialize(
                        socket_reader,
                        &version,
                    )
                    .await?;
                    messages.push(msg);
                }

                let result = MySbTcpContract::NewMessages(NewMessagesModel {
                    topic_id,
                    queue_id,
                    confirmation_id,
                    messages,
                });

                Ok(result)
            }
            ALL_MESSAGES_DELIVERED_CONFIRMATION => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = MySbTcpContract::NewMessagesConfirmation {
                    topic_id,
                    queue_id,
                    confirmation_id,
                };

                Ok(result)
            }
            CREATE_TOPIC_IF_NOT_EXISTS => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;

                let result = MySbTcpContract::CreateTopicIfNotExists { topic_id };

                Ok(result)
            }

            REJECT => {
                let message =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let result = MySbTcpContract::Reject { message };
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

                let result = MySbTcpContract::PacketVersions { packet_versions };

                Ok(result)
            }

            ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION => {
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;

                let result = MySbTcpContract::AllMessagesConfirmedAsFail {
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

                let result = MySbTcpContract::ConfirmSomeMessagesAsOk {
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

                let result = MySbTcpContract::IntermediaryConfirm {
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                };

                Ok(result)
            }

            NODE_GREETING => {
                let name =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let protocol_version = socket_reader.read_i32().await?;
                Ok(MySbTcpContract::NodeGreeting {
                    name,
                    protocol_version,
                })
            }

            NODE_PUBLISH => {
                let sequence_number = socket_reader.read_i64().await?;
                let topics_count = socket_reader.read_i32().await? as usize;
                let mut topics = Vec::with_capacity(topics_count);
                for _ in 0..topics_count {
                    let topic_id =
                        crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                    let persist_immediately = socket_reader.read_bool().await?;
                    let messages_count = socket_reader.read_i32().await? as usize;
                    let mut data_to_publish = Vec::with_capacity(messages_count);
                    for _ in 0..messages_count {
                        let headers =
                            crate::tcp_serializers::message_headers::deserialize(socket_reader)
                                .await?;
                        let content = socket_reader.read_byte_array().await?;
                        data_to_publish.push(MessageToPublish { headers, content });
                    }
                    topics.push(NodeTopicPublish {
                        topic_id,
                        persist_immediately,
                        data_to_publish,
                    });
                }
                Ok(MySbTcpContract::NodePublish {
                    sequence_number,
                    topics,
                })
            }

            NODE_PUBLISH_RESPONSE => {
                let sequence_number = socket_reader.read_i64().await?;
                let topics_count = socket_reader.read_i32().await? as usize;
                let mut topics = Vec::with_capacity(topics_count);
                for _ in 0..topics_count {
                    let topic_id =
                        crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                    let first_message_id = socket_reader.read_i64().await?;
                    let last_message_id = socket_reader.read_i64().await?;
                    topics.push(NodeTopicAck {
                        topic_id,
                        first_message_id,
                        last_message_id,
                    });
                }
                Ok(MySbTcpContract::NodePublishResponse {
                    sequence_number,
                    topics,
                })
            }

            NODE_SUBSCRIBE => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_type = TopicQueueType::from_u8(socket_reader.read_byte().await?);
                Ok(MySbTcpContract::NodeSubscribe {
                    virtual_subscriber_id,
                    topic_id,
                    queue_id,
                    queue_type,
                })
            }

            NODE_SUBSCRIBE_RESPONSE => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                Ok(MySbTcpContract::NodeSubscribeResponse {
                    virtual_subscriber_id,
                    topic_id,
                    queue_id,
                })
            }

            NODE_UNSUBSCRIBE => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                Ok(MySbTcpContract::NodeUnsubscribe {
                    virtual_subscriber_id,
                })
            }

            NODE_NEW_MESSAGES => {
                use my_service_bus_abstractions::{MySbMessage, SbMessageHeaders};
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;
                let records_len = socket_reader.read_i32().await? as usize;
                let mut messages = Vec::with_capacity(records_len);
                for _ in 0..records_len {
                    let id = socket_reader.read_i64().await?;
                    let attempt_no = socket_reader.read_i32().await?;
                    let headers =
                        crate::tcp_serializers::message_headers::deserialize(socket_reader).await?;
                    let content = socket_reader.read_byte_array().await?;
                    let _ = SbMessageHeaders::new(); // touch import in case headers above empty
                    messages.push(MySbMessage {
                        id: id.into(),
                        attempt_no,
                        headers,
                        content,
                    });
                }
                Ok(MySbTcpContract::NodeNewMessages {
                    virtual_subscriber_id,
                    new_messages: NewMessagesModel {
                        topic_id,
                        queue_id,
                        confirmation_id,
                        messages,
                    },
                })
            }

            NODE_NEW_MESSAGES_CONFIRMATION => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;
                Ok(MySbTcpContract::NodeNewMessagesConfirmation {
                    virtual_subscriber_id,
                    topic_id,
                    queue_id,
                    confirmation_id,
                })
            }

            NODE_ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;
                Ok(MySbTcpContract::NodeAllMessagesConfirmedAsFail {
                    virtual_subscriber_id,
                    topic_id,
                    queue_id,
                    confirmation_id,
                })
            }

            NODE_CONFIRM_SOME_MESSAGES_AS_OK => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;
                let delivered =
                    crate::tcp_serializers::queue_with_intervals::deserialize(socket_reader)
                        .await?;
                Ok(MySbTcpContract::NodeConfirmSomeMessagesAsOk {
                    virtual_subscriber_id,
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                })
            }

            NODE_INTERMEDIARY_CONFIRM => {
                let virtual_subscriber_id = socket_reader.read_i64().await?;
                let packet_version = socket_reader.read_byte().await?;
                let topic_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let queue_id =
                    crate::tcp_serializers::pascal_string::deserialize(socket_reader).await?;
                let confirmation_id = socket_reader.read_i64().await?;
                let delivered =
                    crate::tcp_serializers::queue_with_intervals::deserialize(socket_reader)
                        .await?;
                Ok(MySbTcpContract::NodeIntermediaryConfirm {
                    virtual_subscriber_id,
                    packet_version,
                    topic_id,
                    queue_id,
                    confirmation_id,
                    delivered,
                })
            }

            _ => Err(ReadingTcpContractFail::InvalidPacketId(packet_no)),
        };

        return result;
    }

    pub fn serialize(
        &self,
        write_buffer: &mut impl TcpWriteBuffer,
        serializer_metadata: &MySbSerializerState,
    ) {
        match self {
            MySbTcpContract::Ping {} => {
                write_buffer.write_byte(PING);
            }
            MySbTcpContract::Pong {} => {
                write_buffer.write_byte(PONG);
            }
            MySbTcpContract::Greeting {
                name,
                protocol_version,
            } => {
                write_buffer.write_byte(GREETING);
                write_buffer.write_pascal_string(name.as_str());
                write_buffer.write_i32(*protocol_version);
            }
            MySbTcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => Self::compile_publish_payload(
                write_buffer,
                topic_id.as_str(),
                *request_id,
                data_to_publish.as_slice(),
                *persist_immediately,
                serializer_metadata.tcp_protocol_version,
            ),
            MySbTcpContract::PublishResponse { request_id } => {
                write_buffer.write_byte(PUBLISH_RESPONSE);
                write_buffer.write_i64(*request_id);
                //crate::tcp_serializers::i64::serialize(&mut result, *request_id);
                //result.into()
            }
            MySbTcpContract::Subscribe {
                topic_id,
                queue_id,
                queue_type,
            } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(SUBSCRIBE);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());

                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());

                write_buffer.write_byte(queue_type.into_u8());
                //crate::tcp_serializers::byte::serialize(&mut result, queue_type.into_u8());
                //result.into()
            }
            MySbTcpContract::SubscribeResponse { topic_id, queue_id } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(SUBSCRIBE_RESPONSE);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());

                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());
                // result.into()
            }
            MySbTcpContract::Raw(payload) => {
                write_buffer.write_slice(payload);
            }
            MySbTcpContract::NewMessages(_) => {
                panic!(
                    "This packet is not used by server. Server uses optimized version of the packet"
                );
            }
            MySbTcpContract::NewMessagesConfirmation {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(ALL_MESSAGES_DELIVERED_CONFIRMATION);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());

                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());

                write_buffer.write_i64(*confirmation_id);
                //crate::tcp_serializers::i64::serialize(&mut result, *confirmation_id);
                // result.into()
            }
            MySbTcpContract::CreateTopicIfNotExists { topic_id } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(CREATE_TOPIC_IF_NOT_EXISTS);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                //result.into()
            }
            MySbTcpContract::IntermediaryConfirm {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(INTERMEDIARY_CONFIRM);
                write_buffer.write_byte(*packet_version);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());

                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());

                write_buffer.write_i64(*confirmation_id);
                //crate::tcp_serializers::i64::serialize(&mut result, *confirmation_id);

                crate::tcp_serializers::queue_with_intervals::serialize(write_buffer, &delivered);
                //result.into()
            }
            MySbTcpContract::PacketVersions { packet_versions } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(PACKET_VERSIONS);

                let data_len = packet_versions.len() as u8;
                write_buffer.write_byte(data_len);
                //crate::tcp_serializers::byte::serialize(&mut result, data_len);

                for kv in packet_versions {
                    write_buffer.write_byte(*kv.0);
                    //crate::tcp_serializers::byte::serialize(&mut result, *kv.0);

                    write_buffer.write_i32(*kv.1);
                    //crate::tcp_serializers::i32::serialize(&mut result, *kv.1);
                }
                // result.into()
            }
            MySbTcpContract::Reject { message } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(REJECT);

                write_buffer.write_pascal_string(message);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, message.as_str());
                //result.into()
            }
            MySbTcpContract::AllMessagesConfirmedAsFail {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());

                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());

                write_buffer.write_i64(*confirmation_id);
                //crate::tcp_serializers::i64::serialize(&mut result, *confirmation_id);
                //result.into()
            }

            MySbTcpContract::ConfirmSomeMessagesAsOk {
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                //let mut result: Vec<u8> = Vec::new();
                write_buffer.write_byte(CONFIRM_SOME_MESSAGES_AS_OK);
                write_buffer.write_byte(*packet_version);

                write_buffer.write_pascal_string(topic_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id.as_str());
                write_buffer.write_pascal_string(queue_id);
                //crate::tcp_serializers::pascal_string::serialize(&mut result, queue_id.as_str());

                write_buffer.write_i64(*confirmation_id);
                //crate::tcp_serializers::i64::serialize(&mut result, *confirmation_id);

                crate::tcp_serializers::queue_with_intervals::serialize(write_buffer, &delivered);
                // result.into()
            }
            MySbTcpContract::NodeGreeting {
                name,
                protocol_version,
            } => {
                write_buffer.write_byte(NODE_GREETING);
                write_buffer.write_pascal_string(name.as_str());
                write_buffer.write_i32(*protocol_version);
            }
            MySbTcpContract::NodePublish {
                sequence_number,
                topics,
            } => {
                write_buffer.write_byte(NODE_PUBLISH);
                write_buffer.write_i64(*sequence_number);
                write_buffer.write_i32(topics.len() as i32);
                for topic in topics {
                    write_buffer.write_pascal_string(topic.topic_id.as_str());
                    write_buffer.write_bool(topic.persist_immediately);
                    // Always serialize messages with v3 layout (headers always
                    // present) — Node packets only exist on protocol >= 3.
                    crate::tcp_serializers::messages_to_publish::serialize_v3(
                        write_buffer,
                        topic.data_to_publish.as_slice(),
                    );
                }
            }
            MySbTcpContract::NodePublishResponse {
                sequence_number,
                topics,
            } => {
                write_buffer.write_byte(NODE_PUBLISH_RESPONSE);
                write_buffer.write_i64(*sequence_number);
                write_buffer.write_i32(topics.len() as i32);
                for ack in topics {
                    write_buffer.write_pascal_string(ack.topic_id.as_str());
                    write_buffer.write_i64(ack.first_message_id);
                    write_buffer.write_i64(ack.last_message_id);
                }
            }
            MySbTcpContract::NodeSubscribe {
                virtual_subscriber_id,
                topic_id,
                queue_id,
                queue_type,
            } => {
                write_buffer.write_byte(NODE_SUBSCRIBE);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
                write_buffer.write_byte(queue_type.into_u8());
            }
            MySbTcpContract::NodeSubscribeResponse {
                virtual_subscriber_id,
                topic_id,
                queue_id,
            } => {
                write_buffer.write_byte(NODE_SUBSCRIBE_RESPONSE);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
            }
            MySbTcpContract::NodeUnsubscribe {
                virtual_subscriber_id,
            } => {
                write_buffer.write_byte(NODE_UNSUBSCRIBE);
                write_buffer.write_i64(*virtual_subscriber_id);
            }
            MySbTcpContract::NodeNewMessages {
                virtual_subscriber_id,
                new_messages,
            } => {
                use my_service_bus_abstractions::MyServiceBusMessage;
                write_buffer.write_byte(NODE_NEW_MESSAGES);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_pascal_string(new_messages.topic_id.as_str());
                write_buffer.write_pascal_string(new_messages.queue_id.as_str());
                write_buffer.write_i64(new_messages.confirmation_id);
                write_buffer.write_i32(new_messages.messages.len() as i32);
                // Always serialize with v3 layout for Node packets — they only
                // exist on protocol >= 3 so headers are always present.
                for msg in &new_messages.messages {
                    write_buffer.write_i64(msg.get_id().get_value());
                    write_buffer.write_i32(msg.get_attempt_no());
                    crate::tcp_serializers::message_headers::serialize(
                        write_buffer,
                        msg.get_headers(),
                    );
                    write_buffer.write_byte_array(msg.get_content());
                }
            }
            MySbTcpContract::NodeNewMessagesConfirmation {
                virtual_subscriber_id,
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                write_buffer.write_byte(NODE_NEW_MESSAGES_CONFIRMATION);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
                write_buffer.write_i64(*confirmation_id);
            }
            MySbTcpContract::NodeAllMessagesConfirmedAsFail {
                virtual_subscriber_id,
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                write_buffer.write_byte(NODE_ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
                write_buffer.write_i64(*confirmation_id);
            }
            MySbTcpContract::NodeConfirmSomeMessagesAsOk {
                virtual_subscriber_id,
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                write_buffer.write_byte(NODE_CONFIRM_SOME_MESSAGES_AS_OK);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_byte(*packet_version);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
                write_buffer.write_i64(*confirmation_id);
                crate::tcp_serializers::queue_with_intervals::serialize(write_buffer, delivered);
            }
            MySbTcpContract::NodeIntermediaryConfirm {
                virtual_subscriber_id,
                packet_version,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                write_buffer.write_byte(NODE_INTERMEDIARY_CONFIRM);
                write_buffer.write_i64(*virtual_subscriber_id);
                write_buffer.write_byte(*packet_version);
                write_buffer.write_pascal_string(topic_id);
                write_buffer.write_pascal_string(queue_id);
                write_buffer.write_i64(*confirmation_id);
                crate::tcp_serializers::queue_with_intervals::serialize(write_buffer, delivered);
            }
        }
    }

    pub fn compile_publish_payload(
        write_buffer: &mut impl TcpWriteBuffer,
        topic_id: &str,
        request_id: i64,
        data_to_publish: &[MessageToPublish],
        persist_immediately: bool,
        tcp_protocol_version: crate::TcpProtocolVersion,
    ) {
        write_buffer.write_byte(PUBLISH);
        write_buffer.write_pascal_string(topic_id);
        //crate::tcp_serializers::pascal_string::serialize(&mut result, topic_id);

        write_buffer.write_i64(request_id);
        //crate::tcp_serializers::i64::serialize(&mut result, request_id);

        crate::tcp_serializers::messages_to_publish::serialize(
            write_buffer,
            &data_to_publish,
            tcp_protocol_version,
        );

        write_buffer.write_bool(persist_immediately);
        //crate::tcp_serializers::bool::serialize(&mut result, persist_immediately);
    }

    pub fn unwrap_as_message(
        self,
        packet_version: crate::PacketProtVer,
    ) -> Result<NewMessagesModel, my_tcp_sockets::socket_reader::ReadingTcpContractFail> {
        match self {
            MySbTcpContract::Raw(payload) => {
                NewMessagesModel::deserialize(payload.as_slice(), &packet_version)
            }
            MySbTcpContract::NewMessages(model) => Ok(model),
            _ => {
                panic!("Invalid packet type: {}", self.as_str());
            }
        }
    }
}

impl my_tcp_sockets::TcpContract for MySbTcpContract {
    fn is_ping(&self) -> bool {
        if let MySbTcpContract::Ping = self {
            return true;
        }

        false
    }

    fn is_pong(&self) -> bool {
        if let MySbTcpContract::Pong = self {
            return true;
        }

        false
    }
}
#[cfg(test)]
mod tests {

    use my_service_bus_abstractions::SbMessageHeaders;
    use my_tcp_sockets::socket_reader::SocketReaderInMem;

    use super::*;

    #[tokio::test]
    async fn test_ping_packet() {
        let tcp_packet = MySbTcpContract::Ping;

        let mut serialized_data = Vec::new();

        let metadata = MySbSerializerState::new(2);
        tcp_packet.serialize(&mut serialized_data, &metadata);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);
        let attr = MySbSerializerState::new(0);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Ping => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_pong_packet() {
        let tcp_packet = MySbTcpContract::Pong;

        let mut serialized_data: Vec<u8> = Vec::new();

        let metadata = MySbSerializerState::new(2);
        tcp_packet.serialize(&mut serialized_data, &metadata);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);
        let attr = MySbSerializerState::new(0);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Pong => {}
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_greeting_packet() {
        let test_app_name = "test_app";
        let test_protocol_version = 2;

        let tcp_packet = MySbTcpContract::Greeting {
            name: test_app_name.to_string(),
            protocol_version: test_protocol_version,
        };

        let mut serialized_data: Vec<u8> = Vec::new();
        let metadata = MySbSerializerState::new(0);
        tcp_packet.serialize(&mut serialized_data, &metadata);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let attr = MySbSerializerState::new(0);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Greeting {
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
            headers: SbMessageHeaders::new(),
        };

        let data_test = vec![message_to_publish];
        let topic_test = String::from("test-topic");
        let persist_test = true;

        let tcp_packet = MySbTcpContract::Publish {
            data_to_publish: data_test,
            persist_immediately: persist_test,
            request_id: request_id_test,
            topic_id: topic_test,
        };
        let attr = MySbSerializerState::new(PROTOCOL_VERSION);

        let mut serialized_data: Vec<u8> = Vec::new();

        tcp_packet.serialize(&mut serialized_data, &attr);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Publish {
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

        let headers = SbMessageHeaders::new()
            .add("key1", "value1")
            .add("key2", "value2");

        let message_to_publish = MessageToPublish {
            content: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0],
            headers,
        };

        let data_test = vec![message_to_publish];
        let topic_test = String::from("test-topic");
        let persist_test = true;

        let tcp_packet = MySbTcpContract::Publish {
            data_to_publish: data_test,
            persist_immediately: persist_test,
            request_id: request_id_test,
            topic_id: topic_test,
        };

        let attr = MySbSerializerState::new(PROTOCOL_VERSION);
        let mut serialized_data: Vec<u8> = Vec::new();
        tcp_packet.serialize(&mut serialized_data, &attr);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Publish {
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

                let mut headers = el0.headers;

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

        let tcp_packet = MySbTcpContract::PublishResponse {
            request_id: request_id_test,
        };

        let attr = MySbSerializerState::new(PROTOCOL_VERSION);

        let mut serialized_data: Vec<u8> = Vec::new();
        tcp_packet.serialize(&mut serialized_data, &attr);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::PublishResponse { request_id } => {
                assert_eq!(request_id_test, request_id);
            }
            _ => {
                panic!("Invalid Packet Type");
            }
        }
    }

    #[tokio::test]
    async fn test_node_publish_roundtrip() {
        let metadata = MySbSerializerState::new(3);

        let msg1 = MessageToPublish {
            content: vec![1, 2, 3],
            headers: SbMessageHeaders::new().add("h1", "v1"),
        };
        let msg2 = MessageToPublish {
            content: vec![4, 5, 6, 7],
            headers: SbMessageHeaders::new(),
        };

        let packet = MySbTcpContract::NodePublish {
            sequence_number: 42,
            topics: vec![
                NodeTopicPublish {
                    topic_id: "topic-a".to_string(),
                    persist_immediately: true,
                    data_to_publish: vec![msg1.clone()],
                },
                NodeTopicPublish {
                    topic_id: "topic-b".to_string(),
                    persist_immediately: false,
                    data_to_publish: vec![msg2.clone()],
                },
            ],
        };

        let mut bytes = Vec::new();
        packet.serialize(&mut bytes, &metadata);

        let mut reader = SocketReaderInMem::new(bytes);
        let result = MySbTcpContract::deserialize(&mut reader, &metadata)
            .await
            .unwrap();

        match result {
            MySbTcpContract::NodePublish {
                sequence_number,
                topics,
            } => {
                assert_eq!(42, sequence_number);
                assert_eq!(2, topics.len());

                assert_eq!("topic-a", topics[0].topic_id);
                assert!(topics[0].persist_immediately);
                assert_eq!(1, topics[0].data_to_publish.len());
                assert_eq!(msg1.content, topics[0].data_to_publish[0].content);
                assert_eq!(
                    "v1",
                    topics[0].data_to_publish[0]
                        .headers
                        .iter()
                        .next()
                        .unwrap()
                        .1
                );

                assert_eq!("topic-b", topics[1].topic_id);
                assert!(!topics[1].persist_immediately);
                assert_eq!(1, topics[1].data_to_publish.len());
                assert_eq!(msg2.content, topics[1].data_to_publish[0].content);
                assert_eq!(0, topics[1].data_to_publish[0].headers.len());
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_node_publish_response_roundtrip() {
        let metadata = MySbSerializerState::new(3);

        let packet = MySbTcpContract::NodePublishResponse {
            sequence_number: 99,
            topics: vec![
                NodeTopicAck {
                    topic_id: "t1".to_string(),
                    first_message_id: 100,
                    last_message_id: 199,
                },
                NodeTopicAck {
                    topic_id: "t2".to_string(),
                    first_message_id: 1000,
                    last_message_id: 1000,
                },
            ],
        };

        let mut bytes = Vec::new();
        packet.serialize(&mut bytes, &metadata);

        let mut reader = SocketReaderInMem::new(bytes);
        let result = MySbTcpContract::deserialize(&mut reader, &metadata)
            .await
            .unwrap();

        match result {
            MySbTcpContract::NodePublishResponse {
                sequence_number,
                topics,
            } => {
                assert_eq!(99, sequence_number);
                assert_eq!(2, topics.len());
                assert_eq!("t1", topics[0].topic_id);
                assert_eq!(100, topics[0].first_message_id);
                assert_eq!(199, topics[0].last_message_id);
                assert_eq!("t2", topics[1].topic_id);
                assert_eq!(1000, topics[1].first_message_id);
                assert_eq!(1000, topics[1].last_message_id);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_node_subscribe_roundtrip() {
        let metadata = MySbSerializerState::new(3);

        let packet = MySbTcpContract::NodeSubscribe {
            virtual_subscriber_id: 7,
            topic_id: "topic".to_string(),
            queue_id: "queue".to_string(),
            queue_type: TopicQueueType::Permanent,
        };

        let mut bytes = Vec::new();
        packet.serialize(&mut bytes, &metadata);
        let mut reader = SocketReaderInMem::new(bytes);
        let result = MySbTcpContract::deserialize(&mut reader, &metadata)
            .await
            .unwrap();

        match result {
            MySbTcpContract::NodeSubscribe {
                virtual_subscriber_id,
                topic_id,
                queue_id,
                queue_type,
            } => {
                assert_eq!(7, virtual_subscriber_id);
                assert_eq!("topic", topic_id);
                assert_eq!("queue", queue_id);
                assert!(matches!(queue_type, TopicQueueType::Permanent));
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[tokio::test]
    async fn test_subscribe_packet() {
        const PROTOCOL_VERSION: i32 = 2;
        let queue_id_test = String::from("queue");
        let topic_id_test = String::from("topic");
        let queue_type_test = TopicQueueType::PermanentWithSingleConnection;

        let tcp_packet = MySbTcpContract::Subscribe {
            queue_id: queue_id_test,
            topic_id: topic_id_test,
            queue_type: queue_type_test,
        };

        let attr = MySbSerializerState::new(PROTOCOL_VERSION.into());
        let mut serialized_data: Vec<u8> = Vec::new();
        tcp_packet.serialize(&mut serialized_data, &attr);

        let mut socket_reader = SocketReaderInMem::new(serialized_data);

        let result = MySbTcpContract::deserialize(&mut socket_reader, &attr)
            .await
            .unwrap();

        match result {
            MySbTcpContract::Subscribe {
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
