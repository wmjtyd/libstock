//! A high-level abstracted ZeroMQ subscriber and
//! publisher methods with [`zmq2`].
//!
//! - [`ZeromqSubscriber`]: Supports [`SyncSubscriber`](super::traits::SyncSubscriber)
//!   and [`AsyncSubscriber`](super::traits::SyncSubscriber).
//! - [`ZeromqPublisher`]: Supports [`SyncSubscriber`](super::traits::SyncSubscriber)
//!   and [`AsyncSubscriber`](super::traits::SyncSubscriber).
//!
//! Note that:
//!
//! - We return [`MessageError`](super::MessageError) instead of
//!   [`ZeromqError`], to maintain the same error
//!   type as the other implementations.
//!
//! # Example
//!
//! ## Direct Example
//!
//! ```
//! use wmjtyd_libstock::message::traits::{Write, Bind};
//! use wmjtyd_libstock::message::zeromq::ZeromqPublisher;
//!
//! let zeromq = ZeromqPublisher::new();
//!
//! if let Ok(mut zeromq) = zeromq {
//!     zeromq.bind("ipc:///tmp/cl-zeromq-new-api-w-a.ipc").ok();
//!     zeromq.write_all(b"Hello World!").ok();
//! }
//! ```
//!
//! ## Abstract Example
//!
//! ```
//! use std::fmt::Debug;
//! use wmjtyd_libstock::message::traits::{Bind, Connect, Subscribe, SyncPublisher, SyncSubscriber};
//!
//! fn abstract_write_function(mut publisher: impl Bind<Err = impl Debug> + SyncPublisher, addr: &str) {
//!     publisher.bind(addr).expect("failed to bind");
//!
//!     loop {
//!         publisher
//!             .write_all(b"TEST Hello, World")
//!             .expect("failed to write");
//!         publisher
//!             .flush()
//!             .expect("failed to flush")
//!     }
//! }
//!
//! fn abstract_read_function<S, E>(mut subscriber: S, addr: &str)
//! where
//!     E: Debug,
//!     S: Connect<Err = E> + SyncSubscriber<Err = E> + Subscribe<Err = E>,
//! {
//!     subscriber.connect(addr).expect("failed to connect");
//!     subscriber.subscribe(b"TEST").expect("failed to subscribe");
//!
//!     let message = subscriber.next().expect("no data inside");
//!     assert_eq!(
//!         message.expect("data receiving failed"),
//!         b"TEST Hello, World"
//!     );
//! }
//!
//! fn zeromq_example() {
//!     const IPC_ADDR: &str = "ipc:///tmp/libstock-zeromq-test.ipc";
//!
//!     use wmjtyd_libstock::message::zeromq::{ZeromqPublisher, ZeromqSubscriber};
//!
//!     let publisher = ZeromqPublisher::new().expect("failed to create publisher");
//!     let subscriber = ZeromqSubscriber::new().expect("failed to create subscriber");
//!
//!     std::thread::spawn(move || abstract_write_function(publisher, IPC_ADDR));
//!     std::thread::spawn(move || abstract_read_function(subscriber, IPC_ADDR))
//!         .join()
//!         .unwrap();
//! }
//!
//! zeromq_example();
//! ```

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

#[cfg(test)]
mod tests {
    use crate as wmjtyd_libstock;

    mod changelog_0_4_0 {
        use super::*;

        #[test]
        fn migrate_to_new_api_write() {
            use wmjtyd_libstock::message::traits::{Bind, Write};
            use wmjtyd_libstock::message::zeromq::ZeromqPublisher;

            let zeromq = ZeromqPublisher::new();

            if let Ok(mut zeromq) = zeromq {
                zeromq
                    .bind("ipc:///tmp/cl-zeromq_migrate_to_new_api_write.ipc")
                    .ok();
                zeromq.write_all(b"Hello World!").ok();
            }
        }

        #[tokio::test]
        async fn migrate_to_new_api_write_async() {
            use wmjtyd_libstock::message::traits::{AsyncWriteExt, Bind};
            use wmjtyd_libstock::message::zeromq::ZeromqPublisher;

            let zeromq = ZeromqPublisher::new();

            if let Ok(mut zeromq) = zeromq {
                zeromq
                    .bind("ipc:///tmp/cl-zeromq_migrate_to_new_api_write_async.ipc")
                    .ok();
                zeromq.write_all(b"Hello World!").await.ok();
            }
        }

        #[test]
        fn new_read_example() {
            /* NOT IN DOC -- BEGIN -- Start a thread to write. */
            std::thread::spawn(|| {
                use wmjtyd_libstock::message::traits::{Bind, Write};
                use wmjtyd_libstock::message::zeromq::ZeromqPublisher;

                let zeromq = ZeromqPublisher::new();

                if let Ok(mut zeromq) = zeromq {
                    zeromq
                        .bind("ipc:///tmp/cl-zeromq_new_read_example.ipc")
                        .ok();
                    loop {
                        zeromq.write_all(b"Hello World!").ok();
                    }
                }
            });
            /* NOT IN DOC -- END -- Start a thread to write. */

            use wmjtyd_libstock::message::traits::{Connect, Read, Subscribe};
            use wmjtyd_libstock::message::zeromq::ZeromqSubscriber;

            let zeromq = ZeromqSubscriber::new();

            if let Ok(mut zeromq) = zeromq {
                zeromq
                    .connect("ipc:///tmp/cl-zeromq_new_read_example.ipc")
                    .ok();
                zeromq.subscribe(b"").ok();

                let mut buf = [0; 12];
                zeromq.read_exact(&mut buf).ok();
                assert_eq!(b"Hello World!", &buf);
            }
        }

        #[tokio::test]
        async fn new_read_async_example() {
            /* NOT IN DOC -- BEGIN -- Start a thread to write. */
            std::thread::spawn(|| {
                use wmjtyd_libstock::message::traits::{Bind, Write};
                use wmjtyd_libstock::message::zeromq::ZeromqPublisher;

                let zeromq = ZeromqPublisher::new();

                if let Ok(mut zeromq) = zeromq {
                    zeromq
                        .bind("ipc:///tmp/cl-zeromq_new_read_async_example.ipc")
                        .ok();
                    loop {
                        zeromq.write_all(b"Hello World!").ok();
                    }
                }
            });
            /* NOT IN DOC -- END -- Start a thread to write. */

            use wmjtyd_libstock::message::traits::{AsyncReadExt, Connect, Subscribe};
            use wmjtyd_libstock::message::zeromq::ZeromqSubscriber;

            let zeromq = ZeromqSubscriber::new();

            if let Ok(mut zeromq) = zeromq {
                zeromq
                    .connect("ipc:///tmp/cl-zeromq_new_read_async_example.ipc")
                    .ok();
                zeromq.subscribe(b"").ok();

                let mut buf = [0; 12];
                zeromq.read_exact(&mut buf).await.ok();
                assert_eq!(b"Hello World!", &buf);
            }
        }
    }
}
