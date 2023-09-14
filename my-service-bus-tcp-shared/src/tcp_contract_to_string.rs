use super::TcpContract;

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

impl TcpContract {
    pub fn to_string(&self) -> &'static str {
        match self {
            TcpContract::Ping => PING_NAME,
            TcpContract::Pong => PONG_NAME,
            TcpContract::Greeting {
                name: _,
                protocol_version: _,
            } => GREETING_NAME,
            TcpContract::Publish {
                topic_id: _,
                request_id: _,
                persist_immediately: _,
                data_to_publish: _,
            } => PUBLISH_NAME,
            TcpContract::PublishResponse { request_id: _ } => PUBLISH_RESPONSE_NAME,
            TcpContract::Subscribe {
                topic_id: _,
                queue_id: _,
                queue_type: _,
            } => SUBSCRIBE_NAME,
            TcpContract::SubscribeResponse {
                topic_id: _,
                queue_id: _,
            } => SUBSCRIBER_RESPONSE,
            TcpContract::NewMessages {
                topic_id: _,
                messages: _,
                confirmation_id: _,
                queue_id: _,
            } => NEW_MESSAGES,
            TcpContract::Raw(_) => RAW_PAYLOAD,
            TcpContract::NewMessagesConfirmation {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
            } => NEW_MESSAGES_CONFIRMATION,
            TcpContract::CreateTopicIfNotExists { topic_id: _ } => CREATE_TOPIC_IF_EXIST,
            TcpContract::IntermediaryConfirm {
                packet_version: _,
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                delivered: _,
            } => INTERMEDIARY_CONFIRM,
            TcpContract::PacketVersions { packet_versions: _ } => PACKET_VERSIONS,
            TcpContract::Reject { message: _ } => REJECT,
            TcpContract::AllMessagesConfirmedAsFail {
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
            } => ALL_MESSAGES_CONFIRMED_AS_FAIL,
            TcpContract::ConfirmSomeMessagesAsOk {
                packet_version: _,
                topic_id: _,
                queue_id: _,
                confirmation_id: _,
                delivered: _,
            } => CONFIRM_SOME_MESSAGES_AS_OK,
        }
    }
}
