use serde::{Deserialize, Serialize};

/// Handshake request sent by client after the initial connection.
/// Handshake requests are used to negotiate the message type
///
/// This message is *always* json as per Hub protocol spec
#[derive(Debug, Serialize, Deserialize)]
struct HandshakeRequest {
    protocol: &'static str,
    version: u8,
}

impl HandshakeRequest {
    /// Creates a new HandshakeRequest object
    ///
    /// The protocol is temporarily hardcoded for ease of testing
    pub const fn new() -> Self {
        HandshakeRequest {
            protocol: "json",
            version: 1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
/// Handshake response sent by the server to acknowledge the handshake request
///
/// Returns an error if the handshake failed
struct HandshakeResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional error message only returned on protocol mismatches
    error: Option<String>,
}
