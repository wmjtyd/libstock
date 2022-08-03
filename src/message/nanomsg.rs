//! A high-level abstracted nanomsg subscriber and
//! publisher methods with [`nanomsg`].
//!
//! - [`NanomsgSubscriber`]: Supports [`SyncSubscriber`](super::traits::SyncSubscriber)
//!   and [`AsyncSubscriber`](super::traits::SyncSubscriber).
//! - [`NanomsgPublisher`]: Supports [`SyncSubscriber`](super::traits::SyncSubscriber)
//!   and [`AsyncSubscriber`](super::traits::SyncSubscriber).
//!
//! Note that:
//!
//! - We return [`MessageError`](super::MessageError) instead of
//!   [`NanomsgError`], to maintain the same error
//!   type as the other implementations.
//!
//! # Example
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
//! fn nanomsg_example() {
//!     const IPC_ADDR: &str = "ipc:///tmp/libstock-nanomsg-test.ipc";
//!
//!     use wmjtyd_libstock::message::nanomsg::{NanomsgPublisher, NanomsgSubscriber};
//!
//!     let publisher = NanomsgPublisher::new().expect("failed to create publisher");
//!     let subscriber = NanomsgSubscriber::new().expect("failed to create subscriber");
//!
//!     std::thread::spawn(move || abstract_write_function(publisher, IPC_ADDR));
//!     std::thread::spawn(move || abstract_read_function(subscriber, IPC_ADDR))
//!         .join()
//!         .unwrap();
//! }
//!
//! nanomsg_example();
//! ```

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

#[cfg(test)]
mod tests {
    use crate as wmjtyd_libstock;

    mod changelog_0_4_0 {
        use super::*;

        /**
        API in old days:

        ```compile_fail
        use std::io::Write;
        use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};

        let nanomsg = Nanomsg::new("ipc:///tmp/cl-nanomsg-old-api-w.ipc", NanomsgProtocol::Pub);

        if let Ok(mut nanomsg) = nanomsg {
            nanomsg.write_all(b"Hello World!").ok();
        }
        ```
        */
        #[test]
        fn migrate_to_new_api_write() {
            use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
            use wmjtyd_libstock::message::traits::{Bind, Write};

            let nanomsg = NanomsgPublisher::new();

            if let Ok(mut nanomsg) = nanomsg {
                nanomsg
                    .bind("ipc:///tmp/cl-nanomsg_migrate_to_new_api_write.ipc")
                    .ok();
                nanomsg.write_all(b"Hello World!").ok();
            }
        }

        #[tokio::test]
        async fn migrate_to_new_api_write_async() {
            use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
            use wmjtyd_libstock::message::traits::{AsyncWriteExt, Bind};

            let nanomsg = NanomsgPublisher::new();

            if let Ok(mut nanomsg) = nanomsg {
                nanomsg
                    .bind("ipc:///tmp/cl-nanomsg_migrate_to_new_api_write_async.ipc")
                    .ok();
                nanomsg.write_all(b"Hello World!").await.ok();
            }
        }

        #[test]
        fn new_read_example() {
            /* NOT IN DOC -- BEGIN -- Start a thread to write. */
            std::thread::spawn(|| {
                use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
                use wmjtyd_libstock::message::traits::{Bind, Write};

                let nanomsg = NanomsgPublisher::new();

                if let Ok(mut nanomsg) = nanomsg {
                    nanomsg
                        .bind("ipc:///tmp/cl-nanomsg_new_read_example.ipc")
                        .ok();

                    loop {
                        nanomsg.write_all(b"Hello World!").ok();
                    }
                }
            });
            /* NOT IN DOC -- END -- Start a thread to write. */

            use wmjtyd_libstock::message::nanomsg::NanomsgSubscriber;
            use wmjtyd_libstock::message::traits::{Connect, Read, Subscribe};

            let nanomsg = NanomsgSubscriber::new();

            if let Ok(mut nanomsg) = nanomsg {
                nanomsg
                    .connect("ipc:///tmp/cl-nanomsg_new_read_example.ipc")
                    .ok();
                nanomsg.subscribe(b"").ok();

                let mut buf = [0; 12];
                nanomsg.read_exact(&mut buf).ok();
                assert_eq!(b"Hello World!", &buf);
            }
        }

        #[tokio::test]
        async fn new_read_async_example() {
            /* NOT IN DOC -- BEGIN -- Start a thread to write. */
            std::thread::spawn(|| {
                use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
                use wmjtyd_libstock::message::traits::{Bind, Write};

                let nanomsg = NanomsgPublisher::new();

                if let Ok(mut nanomsg) = nanomsg {
                    nanomsg
                        .bind("ipc:///tmp/cl-nanomsg-new_read_async_example.ipc")
                        .ok();

                    loop {
                        nanomsg.write_all(b"Hello World!").ok();
                    }
                }
            });
            /* NOT IN DOC -- END -- Start a thread to write. */

            use wmjtyd_libstock::message::nanomsg::NanomsgSubscriber;
            use wmjtyd_libstock::message::traits::{AsyncReadExt, Connect, Subscribe};

            let nanomsg = NanomsgSubscriber::new();

            if let Ok(mut nanomsg) = nanomsg {
                nanomsg
                    .connect("ipc:///tmp/cl-nanomsg-new_read_async_example.ipc")
                    .ok();
                nanomsg.subscribe(b"").ok();

                let mut buf = [0; 12];
                nanomsg.read_exact(&mut buf).await.ok();
                assert_eq!(b"Hello World!", &buf);
            }
        }
    }
}
