//! The message exchange methods and utilities.

use std::io::Read;

#[cfg(feature = "nanomsg")]
pub mod nanomsg;

#[cfg(feature = "zeromq")]
pub mod zeromq;

/// The trait for implementing the subscriber of a topic.
pub trait Subscribe: Read {
    /// The return value of `subscribe()`
    type Result;

    /// Subscribe a topic.
    fn subscribe(&mut self, topic: &str) -> Self::Result;
}

/// The trait for implementing the subscriber of a topic.
#[async_trait::async_trait]
pub trait AsyncSubscribe: tokio::io::AsyncRead {
    /// The return value of `subscribe()`
    type Result;

    /// Subscribe a topic.
    async fn subscribe(&mut self, topic: &str) -> Self::Result;
}
