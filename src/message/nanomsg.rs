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
                nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-w.ipc").ok();
                nanomsg.write_all(b"Hello World!").ok();
            }
        }

        #[tokio::test]
        async fn migrate_to_new_api_write_async() {
            use wmjtyd_libstock::message::nanomsg::NanomsgPublisher;
            use wmjtyd_libstock::message::traits::{AsyncWriteExt, Bind};

            let nanomsg = NanomsgPublisher::new();

            if let Ok(mut nanomsg) = nanomsg {
                nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-w-a.ipc").ok();
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
                    nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-r.ipc").ok();

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
                nanomsg.connect("ipc:///tmp/cl-nanomsg-new-api-r.ipc").ok();
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
                    nanomsg.bind("ipc:///tmp/cl-nanomsg-new-api-r.ipc").ok();

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
                nanomsg.connect("ipc:///tmp/cl-nanomsg-new-api-r.ipc").ok();
                nanomsg.subscribe(b"").ok();

                let mut buf = [0; 12];
                nanomsg.read_exact(&mut buf).await.ok();
                assert_eq!(b"Hello World!", &buf);
            }
        }
    }
}
