use serde::{Deserialize, Serialize};

use crate::protocol::message_type::{ExactMessageType, MessageTypeEnum};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Close {
    /// Message type, should always be MessageType::Close here
    #[serde(rename = "type")]
    message_type: ExactMessageType<{ MessageTypeEnum::CLOSE }>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Ack {
    /// Message type, should always be MessageType::Ack here
    #[serde(rename = "type")]
    message_type: ExactMessageType<{ MessageTypeEnum::ACK }>,
    /// Number, specifying the amount of messages ack'd
    sequence_id: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    /// Message type, should always be MessageType::Sequence here
    #[serde(rename = "type")]
    message_type: ExactMessageType<{ MessageTypeEnum::SEQUENCE }>,
    /// Number, specifying the new starting message ID
    ///
    /// Only sent on reconnects
    sequence_id: Option<u32>,
}
