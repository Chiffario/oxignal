use serde::{Deserialize, Serialize};

use crate::protocol::message_type::{ExactMessageType, MessageTypeEnum};

#[derive(Debug, Serialize, Deserialize)]
/// Ping message
pub struct PingMessage {
    #[serde(rename = "type")]
    /// Message type. Must be set to 6 for Ping
    message_type: ExactMessageType<{ MessageTypeEnum::PING }>,
}

impl PingMessage {
    pub const fn new() -> Self {
        Self {
            message_type: ExactMessageType::<{ MessageTypeEnum::PING }>,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::ping::PingMessage;

    #[test]
    fn test_deserialize() {
        let value = r#"{ "type": 6 }"#;
        serde_json::from_str::<PingMessage>(value).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_deserialize() {
        let value = r#"{ "type": 69 }"#;
        serde_json::from_str::<PingMessage>(value).unwrap();
    }
}
