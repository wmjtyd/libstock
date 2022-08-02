//! A high-level abstracted nanomsg subscriber and
//! publisher methods with [`nanomsg`].
//!
//! Note that:
//!
//! - We return [`MessageError`](super::MessageError) instead of
//!   [`NanomsgError`], to maintain the same error
//!   type as the other implementations.

mod common;
mod publisher;
mod subscriber;

pub use publisher::NanomsgPublisher;
pub use subscriber::NanomsgSubscriber;

/// The errors of [`Nanomsg`](self).
#[derive(thiserror::Error, Debug)]
pub enum NanomsgError {
    /// When we can't create socket.
    #[error("Unable to create socket: {0}")]
    CreateSocketFailed(nanomsg::Error),

    /// When we can't connect an address.
    #[error("Failed to connect to an address: {0}")]
    ConnectFailed(nanomsg::Error),

    /// When we can't disconnect from an adress.
    #[error("Failed to disconnect from an address: {0}")]
    DisconnectFailed(nanomsg::Error),

    /// When we can't bind to an adress.
    #[error("Failed to bind to an address: {0}")]
    BindFailed(nanomsg::Error),

    /// When we can't unbind an adress.
    #[error("Failed to unbind from an address: {0}")]
    UnbindFailed(nanomsg::Error),

    /// When the endpoint to unbind is not found.
    #[error("No such an endpoint: {0}")]
    NoSuchEndpoint(String),

    /// When the `read()` operation failed.
    #[error("Failed to read: {0}")]
    ReadFailed(std::io::Error),

    /// When the `write()` operation failed.
    #[error("Failed to write: {0}")]
    WriteFailed(std::io::Error),

    /// When the `.subscribe()` operation failed.
    #[error("Failed to subscribe: {0}")]
    SubscribeFailed(nanomsg::Error),

    /// When the `.unsubscribe()` operation failed.
    #[error("Failed to unsubscribe: {0}")]
    UnsubscribeFailed(nanomsg::Error),
}

/// The result type of [`Nanomsg`](self).
pub type NanomsgResult<T> = Result<T, NanomsgError>;
