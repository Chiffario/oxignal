use serde::{Deserialize, Serialize};

use crate::{ExactMessageType, MessageTypeEnum};

#[derive(Debug, Serialize, Deserialize)]
/// Ping message
pub struct Ping {
    #[serde(rename = "type")]
    /// Message type. Must be set to 6 for Ping
    message_type: ExactMessageType<{ MessageTypeEnum::PING }>,
}

impl Ping {
    /// Creates a ping message
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::Ping;
    /// let ping = Ping::new();
    /// # assert_eq!(serde_json::json!({ "type": 6 }), serde_json::to_value(ping).unwrap());
    /// ```
    pub const fn new() -> Self {
        Self {
            message_type: ExactMessageType,
        }
    }
}

impl Default for Ping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{MessageType, Ping};

    #[test]
    fn test_deserialize_concrete() {
        let value = serde_json::json!({ "type": 6 });
        serde_json::from_value::<Ping>(value).unwrap();
    }

    #[test]
    fn test_deserialize_variant() {
        let value = serde_json::json!({ "type": 6 });
        let variant: MessageType<(), ()> = serde_json::from_value(value).unwrap();
        assert!(matches!(variant, MessageType::Ping(_)))
    }

    #[test]
    #[should_panic]
    fn test_deserialize_concrete_invalid() {
        let value = serde_json::json!({ "type": 69 });
        serde_json::from_value::<Ping>(value).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_deserialize_variant_invalid() {
        let value = serde_json::json!({ "type": 69 });
        serde_json::from_value::<MessageType<(), ()>>(value).unwrap();
    }

    #[test]
    fn test_serialize_concrete() {
        let ping = serde_json::to_value(Ping::new()).unwrap();
        let expected = serde_json::json!({ "type": 6 });
        assert_eq!(ping, expected);
    }

    #[test]
    fn test_serialize_variant() {
        let ping = serde_json::to_value(MessageType::Ping::<(), ()>(Ping::new())).unwrap();
        let expected = serde_json::json!({ "type": 6 });
        assert_eq!(ping, expected);
    }
}
