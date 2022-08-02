//! The subscriber and publisher encapluation of libstock.
//!
//! The core concept of this module is *traits*. We abstracted
//! the subscriber and publisher to the [`Subscriber`](traits::Subscriber)
//! and [`Publisher`](traits::Publisher) trait.
//!
//! We officially supports the following implementation of
//! subscribers and publishers:
//!
//! - [`nanomsg`]: Based on [`::nanomsg`] crate.
//!   - [`nanomsg::NanomsgSubscriber`]: Supports [`traits::SyncSubscriber`]
//!     and [`traits::AsyncSubscriber`].
//!   - [`nanomsg::NanomsgPublisher`]: Supports [`traits::SyncSubscriber`]
//!     and [`traits::AsyncPublisher`].
//! - [`zeromq`]: Based on [`zmq2`] crate.
//!   - [`zeromq::ZeromqSubscriber`]: Supports [`traits::SyncSubscriber`]
//!     and [`traits::AsyncSubscriber`].
//!   - [`zeromq::ZeromqPublisher`]: Supports [`traits::SyncSubscriber`]
//!     and [`traits::AsyncPublisher`].
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
//! nanomsg_example();
//! zeromq_example();
//! ```

pub mod traits;

#[cfg(feature = "nanomsg")]
pub mod nanomsg;

#[cfg(feature = "zeromq")]
pub mod zeromq;

use std::fmt::Debug;

use self::nanomsg::NanomsgError;
use self::zeromq::ZeromqError;

#[derive(thiserror::Error, Debug)]
pub enum MessageError {
    #[error("Nanomsg error: {0}")]
    NanomsgError(#[from] NanomsgError),

    #[error("ZeroMQ error: {0}")]
    ZeromqError(#[from] ZeromqError),
}

pub type MessageResult<T> = Result<T, MessageError>;

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use super::traits::{
        AsyncPublisher,
        AsyncSubscriber,
        Bind,
        Connect,
        Subscribe,
        SyncPublisher,
        SyncSubscriber,
    };

    fn abstract_write_function(
        mut publisher: impl Bind<Err = impl Debug> + SyncPublisher,
        addr: &str,
    ) {
        publisher.bind(addr).expect("failed to bind");

        loop {
            publisher
                .write_all(b"TEST Hello, World")
                .expect("failed to write");
            publisher.flush().expect("failed to flush")
        }
    }

    async fn abstract_async_write_function(
        mut publisher: impl Bind<Err = impl Debug> + AsyncPublisher,
        addr: &str,
    ) {
        use tokio::io::AsyncWriteExt;

        publisher.bind(addr).expect("failed to bind");

        let mut publisher = Box::pin(publisher);

        loop {
            publisher
                .write_all(b"TEST Hello, World")
                .await
                .expect("failed to write");
            publisher.flush().await.expect("failed to flush");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    async fn abstract_async_read_function<E, S>(mut subscriber: S, addr: &str)
    where
        E: Debug,
        S: Connect<Err = E> + AsyncSubscriber<Err = E> + Subscribe<Err = E>,
    {
        use futures::StreamExt;
        use tokio::io::AsyncReadExt;

        subscriber.connect(addr).expect("failed to connect");
        subscriber.subscribe(b"TEST").expect("failed to subscribe");

        let mut subscriber = Box::pin(subscriber);

        let response = subscriber.next().await;
        // let mut buf = [0u8; 17];
        // let message_size = subscriber.read(&mut buf).await.expect("failed to read");

        assert_eq!(response.unwrap().unwrap(), b"TEST Hello, World");
        // assert_eq!(
        //     &buf,
        //     b"TEST Hello, World"
        // );
        // assert_eq!(
        //     message_size,
        //     17
        // )
    }

    fn abstract_read_function<S, E>(mut subscriber: S, addr: &str)
    where
        E: Debug,
        S: Connect<Err = E> + SyncSubscriber<Err = E> + Subscribe<Err = E>,
    {
        subscriber.connect(addr).expect("failed to connect");
        subscriber.subscribe(b"TEST").expect("failed to subscribe");

        let message = subscriber.next().expect("no data inside");
        assert_eq!(
            message.expect("data receiving failed"),
            b"TEST Hello, World"
        );
    }

    mod nanomsg {
        use super::super::nanomsg::{NanomsgPublisher, NanomsgSubscriber};
        use super::*;

        #[test]
        fn sync_v() {
            const IPC_ADDR: &str = "ipc:///tmp/libstock-nanomsg-test.ipc";
            let publisher = NanomsgPublisher::new().expect("failed to create publisher");
            let subscriber = NanomsgSubscriber::new().expect("failed to create subscriber");

            std::thread::spawn(move || abstract_write_function(publisher, IPC_ADDR));
            std::thread::spawn(move || abstract_read_function(subscriber, IPC_ADDR))
                .join()
                .unwrap();
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn async_v() {
            const IPC_ADDR: &str = "ipc:///tmp/libstock-nanomsg-test-async.ipc";
            let publisher = NanomsgPublisher::new().expect("failed to create publisher");
            let subscriber = NanomsgSubscriber::new().expect("failed to create subscriber");

            let publisher_thread =
                tokio::task::spawn(abstract_async_write_function(publisher, IPC_ADDR));
            tokio::task::spawn(abstract_async_read_function(subscriber, IPC_ADDR))
                .await
                .unwrap();

            publisher_thread.abort();
        }
    }

    mod zeromq {
        use super::super::zeromq::{ZeromqPublisher, ZeromqSubscriber};
        use super::*;

        #[test]
        fn sync() {
            const IPC_ADDR: &str = "ipc:///tmp/libstock-zeromq-test.ipc";
            let publisher = ZeromqPublisher::new().expect("failed to create publisher");
            let subscriber = ZeromqSubscriber::new().expect("failed to create subscriber");

            std::thread::spawn(move || abstract_write_function(publisher, IPC_ADDR));
            std::thread::spawn(move || abstract_read_function(subscriber, IPC_ADDR))
                .join()
                .unwrap();
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn async_v() {
            const IPC_ADDR: &str = "ipc:///tmp/libstock-zeromq-test-async.ipc";

            let publisher = ZeromqPublisher::new().expect("failed to create publisher");
            let subscriber = ZeromqSubscriber::new().expect("failed to create subscriber");

            let publisher_thread =
                tokio::task::spawn(abstract_async_write_function(publisher, IPC_ADDR));
            tokio::task::spawn(abstract_async_read_function(subscriber, IPC_ADDR))
                .await
                .unwrap();

            publisher_thread.abort();
        }
    }
}
