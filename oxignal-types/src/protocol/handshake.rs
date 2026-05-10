use serde::{Deserialize, Serialize};

/// Handshake request sent by client after the initial connection.
/// Handshake requests are used to negotiate the message type
///
/// This message is *always* json as per Hub protocol spec
#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeRequest {
    protocol: String,
    version: u8,
}

impl HandshakeRequest {
    /// Creates a new HandshakeRequest object
    ///
    /// The protocol is temporarily hardcoded to json, as currently there
    /// are no plans to support MessagePack
    ///
    /// # Example
    /// ```
    /// # use oxignal_types::HandshakeRequest;
    /// # use serde_json;
    /// let handshake_request = HandshakeRequest::new();
    /// # assert_eq!(serde_json::to_value(handshake_request).unwrap(), serde_json::json!({ "protocol": "json", "version": 1}));
    /// ```
    pub fn new() -> Self {
        HandshakeRequest {
            protocol: "json".to_owned(),
            version: 1,
        }
    }
}

impl Default for HandshakeRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Handshake response sent by the server to acknowledge the handshake request
///
/// Returns an error if the handshake failed
pub struct HandshakeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional error message only returned on protocol mismatches
    pub error: Option<String>,
}
// no `new()` for HandshakeResponse as this is a client library for now
