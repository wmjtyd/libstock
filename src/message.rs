//! The message exchange methods and utilities.

pub mod traits;

#[cfg(feature = "nanomsg")]
pub mod nanomsg;

#[cfg(feature = "zeromq")]
pub mod zeromq;

use self::{zeromq::ZeromqError, nanomsg::NanomsgError};

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("Nanomsg error: {0}")]
    NanomsgError(#[from] NanomsgError),

    #[error("ZeroMQ error: {0}")]
    ZeromqError(#[from] ZeromqError),
}

pub type MessageResult<T> = Result<T, MessageError>;
