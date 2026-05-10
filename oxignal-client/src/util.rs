use std::{collections::VecDeque, fmt::Debug};

use apply::Also;
use futures::{SinkExt, StreamExt};
use serde_core::Deserialize;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

use oxignal_types::SignalrMessage;

const RECORD_SEPARATOR: char = '\u{001e}';
type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// As per HubProtocol documentation, every JSON frame must be separated by a 0x1E.
/// Since doing this in place is stupid, this helper is handy
///
/// Bounds use SignalrMessage because this prevents sending junk over the wire
async fn frame_to_string_with_separator(
    value: &impl SignalrMessage,
) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string(value)?.also(|p| p.push(RECORD_SEPARATOR)))
}

/// Writes a serializable object to the socket.
///
/// Message termination is handled within the function and
/// should not be done externally, even though the unnecessary separators
/// should not affect the flow of the server
pub async fn write_frame<S>(
    socket: &mut Socket,
    value: &S,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: SignalrMessage,
{
    let payload = frame_to_string_with_separator(value).await?;
    tracing::debug!(frame = payload, "Writing frame");
    socket
        .send(Message::Text(payload.into()))
        .await
        .map_err(Into::into)
}

pub async fn read_next_frame<T>(
    socket: &mut Socket,
    frames: &mut VecDeque<String>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: oxignal_types::SignalrMessage,
{
    if let Some(frame) = frames.pop_front() {
        return frame_to_concrete(frame).map_err(Into::into);
    }

    loop {
        let message = socket.next().await.ok_or("websocket closed")??;

        match message {
            Message::Text(text) => {
                queue_signalr_frames(text.as_str(), frames);
            }
            Message::Binary(bytes) => {
                let text = String::from_utf8(bytes.to_vec())?;
                queue_signalr_frames(text.as_str(), frames);
            }
            Message::Ping(bytes) => {
                socket.send(Message::Pong(bytes)).await?;
            }
            Message::Pong(_) => {}
            Message::Frame(_) => {}
            Message::Close(frame) => {
                return Err(format!("websocket closed by peer: {frame:?}").into());
            }
        }
        if let Some(frame) = frames.pop_front() {
            return frame_to_concrete(frame).map_err(Into::into);
        }
    }
}

fn queue_signalr_frames(text: &str, frames: &mut VecDeque<String>) {
    frames.extend(
        text.split(RECORD_SEPARATOR)
            .filter(|part| !part.is_empty())
            .map(ToOwned::to_owned),
    );
}

/// Small helper for conversions to SignalrMessage types
///
/// Not really necessary but it ensures the types typecheck
fn frame_to_concrete<T>(frame: impl AsRef<str>) -> Result<T, serde_json::Error>
where
    T: SignalrMessage,
{
    serde_json::from_str(frame.as_ref())
        .inspect(|r| tracing::trace!(concrete = ?r, "Converting to concrete"))
}
