//! The abstracts (traits) of [`Message`](super).

pub use std::io::{Read, Write};

pub use futures::{Stream, StreamExt};
pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// The trait for implementing the subscriber of a topic.
pub trait Subscribe {
    type Err;

    /// Subscribe a topic.
    fn subscribe(&mut self, topic: &[u8]) -> Result<(), Self::Err>;

    /// Unsubscribe a topic.
    fn unsubscribe(&mut self, topic: &[u8]) -> Result<(), Self::Err>;
}

/// The trait for connecting to the specified URI after configuration.
pub trait Connect {
    type Err;

    /// Connect to the specified URI.
    fn connect(&mut self, uri: &str) -> Result<(), Self::Err>;

    /// Disconnect from the specified URI.
    fn disconnect(&mut self, uri: &str) -> Result<(), Self::Err>;
}

/// The trait for binding to the specified URI after configuration.
pub trait Bind {
    type Err;

    /// Bind to the specified URI.
    fn bind(&mut self, uri: &str) -> Result<(), Self::Err>;

    /// Unbind from the specified URI.
    fn unbind(&mut self, uri: &str) -> Result<(), Self::Err>;
}

/// The trait for indicating that it is a Subscriber in compile-time.
pub trait Subscriber: Connect {}

/// The trait for indicating that it is a Publisher in compile-time.
pub trait Publisher: Bind {}

/// The trait that provides the synchronous reader ([`Read`])
/// and iterator ([`Iterator`]) of [`Subscriber`].
pub trait SyncSubscriber: Read + Iterator<Item = SubscribeStreamItem<Self::Err>> {
    type Err;
}

/// The trait that provides the synchronous writer ([`Write`])
/// of [`Subscriber`].
pub trait SyncPublisher: Write {}

/// The trait that provides the asynchronous reader ([`AsyncRead`])
/// and stream ([`Stream`]) of [`Subscriber`].
pub trait AsyncSubscriber: AsyncRead + Stream<Item = SubscribeStreamItem<Self::Err>> {
    type Err;
}

/// The trait that provides the asynchronous writer ([`AsyncWrite`])
/// of [`Subscriber`].
pub trait AsyncPublisher: AsyncWrite {}

/// The item that a iterator or stream should return.
pub type SubscribeStreamItem<E> = Result<Vec<u8>, E>;
