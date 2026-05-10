use std::{collections::VecDeque, fmt::Debug};

use oxignal_types::{
    Completion, CompletionResult, HandshakeRequest, HandshakeResponse, Invocation,
};
use serde_core::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{self, http::Uri},
};

use crate::util::*;

pub struct SignalrClient {
    socket_inner: WebSocketStream<MaybeTlsStream<TcpStream>>,
    frames_queue: VecDeque<String>,
}

impl SignalrClient {
    pub async fn new(url: Uri) -> Result<Self, tungstenite::Error> {
        tracing::debug!(url = %url, "Connecting to websocket");
        let (socket, _) = connect_async(url).await?;
        tracing::debug!("Successfully connected to websocket");
        Ok(Self {
            socket_inner: socket,
            frames_queue: VecDeque::new(),
        })
    }
    /// Runs a handshake in a kinda-blocking way
    pub async fn handshake(&mut self) -> Result<HandshakeResponse, Box<dyn std::error::Error>> {
        // Write a handshake request frame and send it
        write_frame(&mut self.socket_inner, &HandshakeRequest::new()).await?;

        // Receive a response
        let response =
            read_next_frame::<HandshakeResponse>(&mut self.socket_inner, &mut self.frames_queue)
                .await?;
        Ok(response)
    }

    pub async fn invoke<T>(
        &mut self,
        message: T,
        target: String,
        invocation_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug,
    {
        write_frame(
            &mut self.socket_inner,
            &Invocation::new(invocation_id, target, message),
        )
        .await
    }

    pub async fn wait_for_completion<T>(
        &mut self,
        invocation_id: Option<String>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug,
    {
        let frame =
            read_next_frame::<Completion<T>>(&mut self.socket_inner, &mut self.frames_queue)
                .await?;
        tracing::trace!(completion = ?frame, "Received frame");

        if let Some(id) = invocation_id
            && frame.invocation_id == id
        {
            return match frame.result.unwrap() {
                CompletionResult::Result(r) => Ok(r),
                CompletionResult::Error(e) => Err(e.into()),
            };
        } else {
            Err(String::from("Failed to receive message").into())
        }
    }

    pub async fn wait_for_server_invocation<T>(
        &mut self,
        target: &str,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Serialize + for<'de> Deserialize<'de> + Debug,
    {
        let frame =
            read_next_frame::<Invocation<T>>(&mut self.socket_inner, &mut self.frames_queue)
                .await?;

        if target == &frame.target {
            tracing::debug!(target = ?target, invoked = ?frame.arguments, "matched expected invocation");
        }
        Ok(())
    }
}
