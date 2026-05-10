use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub mod connection;
pub mod handshake;
pub mod invocation;
pub mod message_type;
pub mod ping;

/// Marker-trait to act as a function generic boundary
pub trait SignalrMessage: Serialize + for<'de> Deserialize<'de> + Debug {}

impl<'de> SignalrMessage for connection::Ack {}
impl<'de> SignalrMessage for connection::Close {}
impl<'de> SignalrMessage for connection::Sequence {}

impl<'de> SignalrMessage for handshake::HandshakeRequest {}
impl<'de> SignalrMessage for handshake::HandshakeResponse {}

impl<'de> SignalrMessage for invocation::CancelInvocation {}
impl<T> SignalrMessage for invocation::Completion<T> where
    T: Serialize + for<'de> Deserialize<'de> + Debug
{
}
impl<T> SignalrMessage for invocation::CompletionResult<T> where
    T: Serialize + for<'de> Deserialize<'de> + Debug
{
}
impl<T> SignalrMessage for invocation::Invocation<T> where
    T: Serialize + for<'de> Deserialize<'de> + Debug
{
}

impl<'de> SignalrMessage for ping::Ping {}
