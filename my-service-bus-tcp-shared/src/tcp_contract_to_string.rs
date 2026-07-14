use super::MySbTcpContract;

const PING_NAME: &str = "Ping";
const PONG_NAME: &str = "Pong";
const GREETING_NAME: &str = "Greeting";
const PUBLISH_NAME: &str = "Publish";
const PUBLISH_RESPONSE_NAME: &str = "PublishResponse";
const SUBSCRIBE_NAME: &str = "Subscribe";
const SUBSCRIBER_RESPONSE: &str = "SubscribeResponse";
const NEW_MESSAGES: &str = "NewMessages";
const RAW_PAYLOAD: &str = "RawPayload";

const NEW_MESSAGES_CONFIRMATION: &str = "NewMessagesConfirmation";

const CREATE_TOPIC_IF_EXIST: &str = "CreateTopicIfExists";

const INTERMEDIARY_CONFIRM: &str = "IntermediaryConfirm";

const PACKET_VERSIONS: &str = "PacketVersions";
const REJECT: &str = "Reject";

const ALL_MESSAGES_CONFIRMED_AS_FAIL: &str = "AllMessagesConfirmedAsFail";

const CONFIRM_SOME_MESSAGES_AS_OK: &str = "ConfirmSomeMessagesAsOk";

impl MySbTcpContract {
    pub fn as_str(&self) -> &'static str {
        match self {
            MySbTcpContract::Ping => PING_NAME,
            MySbTcpContract::Pong => PONG_NAME,
            MySbTcpContract::Greeting {
                name: _,
                protocol_version: _,
            } => GREETING_NAME,
            MySbTcpContract::Publish {
                topic_id: _,
                request_id: _,
                persist_immediately: _,
                data_to_publish: _,
            } => PUBLISH_NAME,
            MySbTcpContract::PublishResponse { request_id: _ } => PUBLISH_RESPONSE_NAME,
            MySbTcpContract::Subscribe {
                topic_id: _,
                queue_id: _,
                queue_type: _,
            } => SUBSCRIBE_NAME,
            MySbTcpContract::SubscribeResponse {
                topic_id: _,
                queue_id: _,
            } => SUBSCRIBER_RESPONSE,
            MySbTcpContract::NewMessages(_) => NEW_MESSAGES,
            MySbTcpContract::Raw(_) => RAW_PAYLOAD,
            MySbTcpContract::NewMessagesConfirmation {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
            } => NEW_MESSAGES_CONFIRMATION,
            MySbTcpContract::CreateTopicIfNotExists { topic_id: _ } => CREATE_TOPIC_IF_EXIST,
            MySbTcpContract::IntermediaryConfirm {
                packet_version: _,
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                delivered: _,
            } => INTERMEDIARY_CONFIRM,
            MySbTcpContract::PacketVersions { packet_versions: _ } => PACKET_VERSIONS,
            MySbTcpContract::Reject { message: _ } => REJECT,
            MySbTcpContract::AllMessagesConfirmedAsFail {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
            } => ALL_MESSAGES_CONFIRMED_AS_FAIL,
            MySbTcpContract::ConfirmSomeMessagesAsOk {
                packet_version: _,
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                delivered: _,
            } => CONFIRM_SOME_MESSAGES_AS_OK,
        }
    }
}
