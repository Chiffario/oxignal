use serde::{Deserialize, Serialize};

use crate::protocol::message_type::{ExactMessageType, MessageTypeEnum};

#[derive(Debug, Deserialize)]
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

impl<T: Serialize> Serialize for Invocation<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;

        // Determine the number of fields we'll serialize
        let mut field_count = 2; // type + target are always present
        if self.invocation_id.is_some() {
            field_count += 1;
        }
        if !self.stream_ids.is_empty() {
            field_count += 1;
        }
        field_count += 1; // arguments

        let mut map = serializer.serialize_map(Some(field_count))?;

        map.serialize_entry("type", &self.message_type)?;

        if let Some(ref id) = self.invocation_id {
            map.serialize_entry("invocationId", id)?;
        }

        map.serialize_entry("target", &self.target)?;

        // Serialize arguments: wrap in array if not already an array
        let args_value =
            serde_json::to_value(&self.arguments).map_err(serde::ser::Error::custom)?;

        let args_array = match args_value {
            serde_json::Value::Array(_) => args_value,
            other => serde_json::Value::Array(vec![other]),
        };

        map.serialize_entry("arguments", &args_array)?;

        if !self.stream_ids.is_empty() {
            map.serialize_entry("streamIds", &self.stream_ids)?;
        }

        map.end()
    }
}

impl<T> Invocation<T> {
    /// Creates an instance of `Invocation`
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::Invocation;
    /// let invocation_id = Some(String::from("123"));
    /// let target = String::from("Send");
    /// let arguments = (42u32, String::from("Hello"));
    /// # let json = serde_json::json!({ "type": 1, "invocationId": "123", "target": "Send", "arguments": [42, "Hello"]});
    /// let invocation = Invocation::new(invocation_id, target, arguments);
    /// # assert_eq!(json, serde_json::to_value(invocation).unwrap());
    /// ```
    pub fn new(invocation_id: impl Into<Option<String>>, target: String, arguments: T) -> Self {
        Self {
            message_type: ExactMessageType::<{ MessageTypeEnum::INVOCATION }>,
            invocation_id: invocation_id.into(),
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
    /// ID used to backreference the callee
    pub invocation_id: String,
}

impl CancelInvocation {
    /// Creates an instance of CancelInvocation
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::CancelInvocation;
    /// let cancel = CancelInvocation::new(String::from("123"));
    /// # assert_eq!(serde_json::json!({"type": 5, "invocationId": "123"}), serde_json::to_value(cancel).unwrap());
    pub const fn new(invocation_id: String) -> Self {
        Self {
            message_type: ExactMessageType,
            invocation_id,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CompletionResult<T> {
    Result(T),
    Error(String),
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
    pub invocation_id: String,
    /// Result, returned for non-void methods, ignored if error exists
    #[serde(flatten)]
    pub result: Option<CompletionResult<T>>,
}

impl<T> Completion<T> {
    /// Initializes a normal Completion without a return value.
    /// Usually used for void methods
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::Completion;
    /// let completion: Completion<(u32, String)> = Completion::new(String::from("123"));
    /// # assert_eq!(serde_json::json!({ "type": 3, "invocationId": "123"}), serde_json::to_value(completion).unwrap());
    /// ```
    pub const fn new(invocation_id: String) -> Self {
        Self {
            message_type: ExactMessageType,
            invocation_id,
            result: None,
        }
    }

    /// Initializes a normal Completion
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::Completion;
    /// let completion = Completion::with_result(String::from("123"), (42u32, String::from("Test")));
    /// # assert_eq!(serde_json::json!({ "type": 3, "invocationId": "123", "result": [42, "Test"]}), serde_json::to_value(completion).unwrap());
    /// ```
    pub const fn with_result(invocation_id: String, result: T) -> Self {
        Self {
            message_type: ExactMessageType,
            invocation_id,
            result: Some(CompletionResult::Result(result)),
        }
    }

    /// Initializes a failed Completion.
    /// Used for errors
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::Completion;
    /// let completion: Completion<(u32, String)> = Completion::with_error(String::from("123"), String::from("Oops"));
    /// # assert_eq!(serde_json::json!({ "type": 3, "invocationId": "123", "error": "Oops"}), serde_json::to_value(completion).unwrap());
    /// ```
    pub const fn with_error(invocation_id: String, reason: String) -> Self {
        Self {
            message_type: ExactMessageType,
            invocation_id,
            result: Some(CompletionResult::Error(reason)),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::error;

    use serde::Serialize;

    use crate::protocol::{
        invocation::{CancelInvocation, Completion, Invocation},
        message_type::{ExactMessageType, MessageType},
    };

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

    #[derive(Serialize)]
    struct InnerStruct {
        number: u32,
        value: String,
    }

    #[derive(Serialize)]
    struct OuterStruct {
        inner: InnerStruct,
    }
    #[test]
    fn test_invocation_ser_struct() {
        let valid: Invocation<InnerStruct> = Invocation::new(
            None,
            String::from("Send"),
            InnerStruct {
                number: 123,
                value: String::from("Test"),
            },
        );
        let value = serde_json::to_value(&valid).unwrap();
        let correct = serde_json::json!({"type": 1, "target": "Send", "arguments": [
            {"number": 123, "value": "Test"}]
        });
        assert_eq!(value, correct);
    }

    #[test]
    fn test_invocation_ser_nested_struct() {
        let valid: Invocation<OuterStruct> = Invocation::new(
            None,
            String::from("Send"),
            OuterStruct {
                inner: InnerStruct {
                    number: 123,
                    value: String::from("Test"),
                },
            },
        );
        let value = serde_json::to_value(&valid).unwrap();
        let correct = serde_json::json!({"type": 1, "target": "Send", "arguments": [
            {"inner": {"number": 123, "value": "Test"}}]
        });
        assert_eq!(value, correct);
    }

    #[test]
    fn test_invocation_ser_variant() {
        let valid: Invocation<(i32, String)> = Invocation::new(
            None,
            String::from("Send"),
            (42, String::from("Test Message")),
        );
        let variant: MessageType<(i32, String), ()> =
            MessageType::<(i32, String), ()>::Invocation(valid);
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

    #[test]
    fn test_cancel_invocation_ser() {
        let valid = serde_json::json!({ "type": 5, "invocationId": "123" });
        let _: CancelInvocation = serde_json::from_value(valid).unwrap();
    }
    #[test]
    fn test_completion_de_no_result() {
        let value = serde_json::json!({"type": 3, "invocationId": "123"});
        let result: Completion<u32> = serde_json::from_value(value).unwrap();
        assert!(matches!(result.result, None))
    }

    #[test]
    fn test_completion_de_result() {
        let value = serde_json::json!({"type": 3, "invocationId": "123", "result": 123});
        let result: Completion<u32> = serde_json::from_value(value).unwrap();
        assert!(matches!(
            result.result,
            Some(crate::CompletionResult::Result(123))
        ));
    }

    #[test]
    fn test_completion_de_error() {
        let value = serde_json::json!({"type": 3, "invocationId": "123", "error": "Womp womp"});
        let result: Completion<u32> = serde_json::from_value(value).unwrap();
        let error_string = String::from("Womp womp");
        assert!(matches!(
            result.result,
            Some(crate::CompletionResult::Error(_))
        ));
    }

    #[test]
    fn test_completion_ser_result() {
        let value: Completion<u32> = Completion {
            message_type: ExactMessageType,
            invocation_id: String::from("123"),
            result: Some(crate::CompletionResult::Result(42)),
        };
        let expected = serde_json::json!({
            "type": 3,
            "invocationId": "123",
            "result": 42
        });
        assert_eq!(serde_json::to_value(value).unwrap(), expected);
    }
    #[test]
    fn test_completion_ser_error() {
        let value: Completion<u32> = Completion {
            message_type: ExactMessageType,
            invocation_id: String::from("123"),
            result: Some(crate::CompletionResult::Error(String::from("Womp womp"))),
        };
        let expected = serde_json::json!({
            "type": 3,
            "invocationId": "123",
            "error": String::from("Womp womp")
        });
        assert_eq!(serde_json::to_value(value).unwrap(), expected);
    }
}
