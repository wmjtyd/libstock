//! A high-level abstracted ZeroMQ subscriber and
//! publisher methods with [`zmq2`].
//!
//! Note that:
//!
//! - We return [`MessageError`](super::MessageError) instead of
//!   [`ZeromqError`], to maintain the same error
//!   type as the other implementations.

mod common;
mod publisher;
mod subscriber;

pub use publisher::ZeromqPublisher;
pub use subscriber::ZeromqSubscriber;

/// The errors of [`Zeromq`](self).
#[derive(thiserror::Error, Debug)]
pub enum ZeromqError {
    /// When we failed to create a socket.
    #[error("Unable to create socket: {0}")]
    CreateSocketFailed(zmq2::Error),

    /// When we failed to connect to an address.
    #[error("Failed to connect to an address: {0}")]
    ConnectFailed(zmq2::Error),

    /// When we failed to disconnect from an address.
    #[error("Failed to disconnect from an address: {0}")]
    DisconnectFailed(zmq2::Error),

    /// When we failed to bind to an address.
    #[error("Failed to bind to an address: {0}")]
    BindFailed(zmq2::Error),

    /// When we failed to unbind an address.
    #[error("Failed to unbind from an address: {0}")]
    UnbindFailed(zmq2::Error),

    /// When the `recv()` operation (called by `read()` or whatever) failed.
    #[error("Failed to receive: {0}")]
    RecvFailed(zmq2::Error),

    /// When the `send()` operation (called by `write()` or whatever) failed.
    #[error("Failed to send: {0}")]
    SendFailed(zmq2::Error),

    /// When the `.set_subscribe()` operation failed.
    #[error("Failed to subscribe: {0}")]
    SubscribeFailed(zmq2::Error),

    /// When the `.set_unsubscribe()` operation failed.
    #[error("Failed to unsubscribe: {0}")]
    UnsubscribeFailed(zmq2::Error),
}

/// The result type of [`Zeromq`](self).
pub type ZeromqResult<T> = Result<T, ZeromqError>;
