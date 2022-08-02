//! A basic encap methods of [`nanomsg`].

mod common;
mod publisher;
mod subscriber;

pub use publisher::NanomsgPublisher;
pub use subscriber::NanomsgSubscriber;

#[derive(thiserror::Error, Debug)]
pub enum NanomsgError {
    #[error("Unable to create socket: {0}")]
    CreateSocketFailed(nanomsg::Error),

    #[error("Failed to connect to an address: {0}")]
    ConnectFailed(nanomsg::Error),

    #[error("Failed to disconnect from an address: {0}")]
    DisconnectFailed(nanomsg::Error),

    #[error("Failed to bind to an address: {0}")]
    BindFailed(nanomsg::Error),

    #[error("Failed to unbind from an address: {0}")]
    UnbindFailed(nanomsg::Error),

    #[error("No such an endpoint: {0}")]
    NoSuchEndpoint(String),

    #[error("Failed to read: {0}")]
    ReadFailed(std::io::Error),

    #[error("Failed to write: {0}")]
    WriteFailed(std::io::Error),
}

pub type NanomsgResult<T> = Result<T, NanomsgError>;
