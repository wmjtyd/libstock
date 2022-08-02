//! A basic encap methods of [`zmq`].
//!
//! Note that:
//! - We return [`MessageError`] to maintain the
//!   same error type as the other implementations.

mod common;
mod publisher;
mod subscriber;

pub use subscriber::ZeromqSubscriber;

#[derive(thiserror::Error, Debug)]
pub enum ZeromqError {
    #[error("Unable to create socket: {0}")]
    CreateSocketFailed(zmq2::Error),

    #[error("Failed to connect to an address: {0}")]
    ConnectFailed(zmq2::Error),

    #[error("Failed to disconnect from an address: {0}")]
    DisconnectFailed(zmq2::Error),

    #[error("Failed to bind to an address: {0}")]
    BindFailed(zmq2::Error),

    #[error("Failed to unbind from an address: {0}")]
    UnbindFailed(zmq2::Error),

    #[error("Failed to receive: {0}")]
    RecvFailed(zmq2::Error),

    #[error("Failed to send: {0}")]
    SendFailed(zmq2::Error),
}

pub type ZeromqResult<T> = Result<T, ZeromqError>;
