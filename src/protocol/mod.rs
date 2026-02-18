pub mod handshake;
pub mod message_type;
pub mod ping;

use std::vec;

use serde::{Deserialize, Serialize};

use crate::protocol::message_type::{ExactMessageType, MessageTypeEnum};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invocation<T> {
    #[serde(rename = "type")]
    /// Message type, should always be MessageType::Invocation here
    message_type: ExactMessageType<{ MessageTypeEnum::INVOCATION }>,
    /// ID used for blocking invocations to backreference the callee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation_id: Option<String>,
    /// Target function to be called
    pub target: String,
    /// Function call arguments, must be serializable as they are passed in
    /// as a json object
    pub arguments: T,
    // Array of unique stream IDs consumed by target
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    stream_ids: Vec<String>,
}

impl<T> Invocation<T> {
    fn new(invocation_id: Option<String>, target: String, arguments: T) -> Self {
        Self {
            message_type: ExactMessageType::<{ MessageTypeEnum::INVOCATION }>,
            invocation_id,
            target,
            arguments,
            stream_ids: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CancelInvocation {
    #[serde(rename = "type")]
    /// Message type, should always be MessageType::CancelInvocation here
    message_type: ExactMessageType<{ MessageTypeEnum::CANCEL_INVOCATION }>,
    /// ID used for blocking invocations to backreference the callee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation_id: Option<String>,
}

/// Indicates completion of an Invocation. May return an error
///
/// This message type is absent for `void` methods
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Completion<T> {
    /// Message type, should always be MessageType::Completion here
    #[serde(rename = "type")]
    message_type: ExactMessageType<{ MessageTypeEnum::COMPLETION }>,
    /// ID used for blocking invocations to backreference the callee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invocation_id: Option<String>,
    /// Result value, ignored if error is not null
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// Error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

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

#[cfg(test)]
mod tests {
    use crate::protocol::{Invocation, message_type::MessageType};

    #[test]
    fn test_invocation_de() {
        let valid = r#"
{
    "type": 1,
    "invocationId": "123",
    "target": "Send",
    "arguments": [
        42,
        "Test Message"
    ]
}        "#;
        let data = serde_json::from_str::<Invocation<(u32, String)>>(valid).unwrap();
        assert_eq!(data.invocation_id, Some(String::from("123")));
        assert_eq!(data.target, String::from("Send"));
        assert_eq!(data.arguments.0, 42);
        assert_eq!(data.arguments.1, String::from("Test Message"));
        assert!(data.stream_ids.len() == 0);

        let non_blocking = r#"
{
    "type": 1,
    "target": "Send",
    "arguments": [
        42,
        "Test Message"
    ]
}        "#;

        let data = serde_json::from_str::<Invocation<(u32, String)>>(non_blocking).unwrap();
        assert_eq!(data.invocation_id, None);
        assert_eq!(data.target, String::from("Send"));
        assert_eq!(data.arguments.0, 42);
        assert_eq!(data.arguments.1, String::from("Test Message"));
        assert!(data.stream_ids.len() == 0);

        let with_stream = r#"
{
    "type": 1,
    "invocationId": "123",
    "target": "Send",
    "arguments": [
        42,
        "Test Message"
    ],
    "streamIds": [
        "1"
    ]
}        "#;
        let data = serde_json::from_str::<Invocation<(u32, String)>>(with_stream).unwrap();
        assert_eq!(data.invocation_id, Some(String::from("123")));
        assert_eq!(data.target, String::from("Send"));
        assert_eq!(data.arguments.0, 42);
        assert_eq!(data.arguments.1, String::from("Test Message"));
        assert!(data.stream_ids.len() == 1);
    }

    #[test]
    fn test_invocation_ser_concrete() {
        let valid: Invocation<(i32, String)> = Invocation::new(
            None,
            String::from("Send"),
            (42, String::from("Test Message")),
        );
        let value = serde_json::to_string(&valid).unwrap();
        let correct = r#"{"type":1,"target":"Send","arguments":[42,"Test Message"]}"#;
        assert_eq!(value, correct);
    }

    #[test]
    fn test_invocation_ser_variant() {
        let valid: Invocation<(i32, String)> = Invocation::new(
            None,
            String::from("Send"),
            (42, String::from("Test Message")),
        );
        let variant: MessageType<(i32, String), ()> = MessageType::Invocation(valid);
        let value = serde_json::to_string(&variant).unwrap();
        let correct = r#"{"type":1,"target":"Send","arguments":[42,"Test Message"]}"#;
        assert_eq!(value, correct);
    }

    #[test]
    #[should_panic]
    fn test_invocation_de_argument_type() {
        let invalid = r#"
{
    "type": 1,
    "invocationId": "123",
    "target": "Send",
    "arguments": [
        "Test Message",
        42
    ]
}        "#;
        serde_json::from_str::<Invocation<(u32, String)>>(invalid).unwrap();
    }
}
