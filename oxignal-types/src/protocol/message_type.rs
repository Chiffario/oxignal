use serde::{Deserialize, Serialize, Serializer, de};

use crate::protocol::{
    connection::{Ack, Close, Sequence},
    invocation::{CancelInvocation, Completion, Invocation},
    ping::Ping,
};

macro_rules! implement_variants {
    ($struct_name:ident, $($variant:ident => $name:ident = $value:expr),*) => {
        #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
        pub enum $struct_name {
            $($variant = $value),*
        }

        impl $struct_name {
            $(pub const $name: u8 = Self::$variant as u8;)*
        }

        impl TryFrom<u8> for $struct_name {
            type Error = u8;

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $($struct_name::$name => Ok($struct_name::$variant),)*
                    value => Err(value)
                }
            }
        }
    }
}

implement_variants!(
    MessageTypeEnum,
    Invocation => INVOCATION = 1,
    StreamItem => STREAM_ITEM = 2,
    Completion => COMPLETION = 3,
    StreamInvocation => STREAM_INVOCATION = 4,
    CancelInvocation => CANCEL_INVOCATION = 5,
    Ping => PING = 6,
    Close => CLOSE = 7,
    Ack => ACK = 8,
    Sequence => SEQUENCE = 9
);

#[derive(Debug)]
pub enum MessageType<T, U> {
    Invocation(Invocation<T>),
    Completion(Completion<U>),
    CancelInvocation(CancelInvocation),
    Ping(Ping),
    Close(Close),
    Ack(Ack),
    Sequence(Sequence),
}

impl<T: Serialize, U: Serialize> Serialize for MessageType<T, U> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Due to the tags being internal, serialization should be delegated,
        // as by default not doing so makes an extra external tag
        match self {
            MessageType::Invocation(invocation) => invocation.serialize(serializer),
            MessageType::Completion(completion) => completion.serialize(serializer),
            MessageType::CancelInvocation(cancel_invocation) => {
                cancel_invocation.serialize(serializer)
            }
            MessageType::Ping(ping_message) => ping_message.serialize(serializer),
            MessageType::Close(close) => close.serialize(serializer),
            MessageType::Ack(ack) => ack.serialize(serializer),
            MessageType::Sequence(sequence) => sequence.serialize(serializer),
        }
    }
}

impl<'de, T, U> Deserialize<'de> for MessageType<T, U>
where
    T: for<'d> Deserialize<'d>,
    U: for<'d> Deserialize<'d>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        use serde::de::Error;

        // Not sure how optimal this is, but given there is no clonning,
        // at least there is one less alloc than there could've been :p
        let value = serde_json::Value::deserialize(deserializer)?;
        let peek = value
            .get("type")
            .ok_or_else(|| D::Error::custom("Type tag not found".to_string()))?
            .as_u64()
            .ok_or_else(|| D::Error::custom("Failed to convert to number".to_string()))?;

        match peek {
            1 => Ok(MessageType::Invocation(
                serde_json::from_value::<Invocation<T>>(value).map_err(D::Error::custom)?,
            )),
            2 => todo!(),
            3 => Ok(MessageType::Completion(
                serde_json::from_value::<Completion<U>>(value).map_err(D::Error::custom)?,
            )),
            4 => todo!(),
            5 => Ok(MessageType::CancelInvocation(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            )),
            6 => Ok(MessageType::Ping(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            )),
            7 => Ok(MessageType::Close(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            )),
            8 => Ok(MessageType::Ack(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            )),
            9 => Ok(MessageType::Sequence(
                serde_json::from_value(value).map_err(D::Error::custom)?,
            )),
            t => Err(D::Error::custom(format!("Unknown message type: {}", t))),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ExactMessageType<const T: u8>;

impl<'de, const T: u8> Deserialize<'de> for ExactMessageType<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        if value == T {
            Ok(ExactMessageType)
        } else {
            // Convert the raw u8s back to the enum variant names for a readable error
            let expected = MessageTypeEnum::try_from(T)
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|_| T.to_string());
            let got = MessageTypeEnum::try_from(value)
                .map(|t| format!("{:?}", t))
                .unwrap_or_else(|_| value.to_string());

            Err(de::Error::custom(format!(
                "expected message_type {expected}, got {got}"
            )))
        }
    }
}

impl<const T: u8> Serialize for ExactMessageType<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(T)
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::{
        invocation::Invocation,
        message_type::{ExactMessageType, MessageType, MessageTypeEnum},
    };
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct TestStruct {
        value: ExactMessageType<{ MessageTypeEnum::INVOCATION }>,
    }

    #[test]
    fn test_tags() {
        let correct = r#"
        {
            "type": 1,
            "target": "Send",
            "arguments": [
                42,
                "Test Message"
            ]
        }        "#;
        let message = serde_json::from_str::<MessageType<(u32, String), ()>>(correct).unwrap();
        assert!(matches!(message, MessageType::Invocation(_)));
        serde_json::from_str::<Invocation<(u32, String)>>(correct).unwrap();
    }

    #[test]
    fn test_const_deserialization() {
        let correct = r#"{ "value": 1 }"#;
        serde_json::from_str::<TestStruct>(correct).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_invalid_deserialization() {
        let invalid = r#"{ "value": 5 }"#;
        serde_json::from_str::<TestStruct>(invalid).unwrap();
    }
}
