pub const PING: u8 = 0;
pub const PONG: u8 = 1;
pub const GREETING: u8 = 2;
pub const PUBLISH: u8 = 3;
pub const PUBLISH_RESPONSE: u8 = 4;
pub const SUBSCRIBE: u8 = 5;
pub const SUBSCRIBE_RESPONSE: u8 = 6;
pub const NEW_MESSAGES: u8 = 7;
pub const ALL_MESSAGES_DELIVERED_CONFIRMATION: u8 = 8;
pub const CREATE_TOPIC_IF_NOT_EXISTS: u8 = 9;
//Not Supported const MESSAGES_DELIVERED_AND_NOT_DELIVERED_CONFIRMATION: u8 = 10;
pub const PACKET_VERSIONS: u8 = 11;
pub const REJECT: u8 = 12;
pub const ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION: u8 = 13;
pub const CONFIRM_SOME_MESSAGES_AS_OK: u8 = 14;
pub const INTERMEDIARY_CONFIRM: u8 = 15; //Confirms some messages within Delivery but not complete Delivery

// Node-mode packets: used when a my-service-bus-node sits between clients and
// the master and multiplexes many virtual clients over a single TCP connection.
pub const NODE_GREETING: u8 = 16;
pub const NODE_PUBLISH: u8 = 17;
pub const NODE_PUBLISH_RESPONSE: u8 = 18;
pub const NODE_SUBSCRIBE: u8 = 19;
pub const NODE_SUBSCRIBE_RESPONSE: u8 = 20;
pub const NODE_UNSUBSCRIBE: u8 = 21;
pub const NODE_NEW_MESSAGES: u8 = 22;
pub const NODE_NEW_MESSAGES_CONFIRMATION: u8 = 23;
pub const NODE_CONFIRM_SOME_MESSAGES_AS_OK: u8 = 24;
pub const NODE_ALL_MESSAGES_NOT_DELIVERED_CONFIRMATION: u8 = 25;
pub const NODE_INTERMEDIARY_CONFIRM: u8 = 26;
