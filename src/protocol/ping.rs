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
    use crate::protocol::{message_type::MessageType, ping::PingMessage};

    #[test]
    fn test_deserialize_concrete() {
        let value = r#"{ "type": 6 }"#;
        serde_json::from_str::<PingMessage>(value).unwrap();
    }

    #[test]
    fn test_deserialize_variant() {
        let value = r#"{ "type": 6 }"#;
        let variant: MessageType<(), ()> = serde_json::from_str(value).unwrap();
        assert!(matches!(variant, MessageType::Ping(_)))
    }

    #[test]
    #[should_panic]
    fn test_invalid_deserialize() {
        let value = r#"{ "type": 69 }"#;
        serde_json::from_str::<PingMessage>(value).unwrap();
    }

    #[test]
    fn test_serialize() {
        let ping = serde_json::to_value(PingMessage::new()).unwrap();
        let expected = serde_json::json!({ "type": 6 });
        assert_eq!(ping, expected);
    }
}
