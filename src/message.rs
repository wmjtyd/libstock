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
        use tokio::io::AsyncReadExt;

        subscriber.connect(addr).expect("failed to connect");
        subscriber.subscribe(b"TEST").expect("failed to subscribe");

        let mut subscriber = Box::pin(subscriber);

        let mut buf = [0u8; 17];
        let message_size = subscriber.read(&mut buf).await.expect("failed to read");

        assert_eq!(
            &buf,
            b"TEST Hello, World"
        );
        assert_eq!(
            message_size,
            17
        )
    }

    async fn abstract_async_stream_function<E, S>(mut subscriber: S, addr: &str)
    where
        E: Debug,
        S: Connect<Err = E> + AsyncSubscriber<Err = E> + Subscribe<Err = E>,
    {
        use futures::StreamExt;

        subscriber.connect(addr).expect("failed to connect");
        subscriber.subscribe(b"TEST").expect("failed to subscribe");

        let mut subscriber = Box::pin(subscriber);

        let response = subscriber.next().await
            .expect("no data inside")
            .expect("data receiving failed");

        assert_eq!(response, b"TEST Hello, World");
    }

    fn abstract_read_function<S, E>(mut subscriber: S, addr: &str)
    where
        E: Debug,
        S: Connect<Err = E> + SyncSubscriber<Err = E> + Subscribe<Err = E>,
    {
        subscriber.connect(addr).expect("failed to connect");
        subscriber.subscribe(b"TEST").expect("failed to subscribe");

        let mut buf = [0; 17];
        subscriber.read_exact(&mut buf)
            .expect("data retriving failed");

        assert_eq!(
            &buf[..],
            b"TEST Hello, World"
        );
    }

    fn abstract_iter_function<S, E>(mut subscriber: S, addr: &str)
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

    macro_rules! build_test {
        (
            sync =>
            func_name = $func_name:ident,
            publisher = $publisher:ident,
            subscriber = $subscriber:ident,
            read_abs = $read_abs:ident,
        ) => {
            #[test]
            fn $func_name() {
                const IPC_ADDR: &str = concat!("ipc:///tmp/libstock_", stringify!($read_abs), stringify!($subscriber), ".ipc");
                let publisher = $publisher::new().expect("failed to create publisher");
                let subscriber = $subscriber::new().expect("failed to create subscriber");

                std::thread::spawn(move || abstract_write_function(publisher, IPC_ADDR));
                std::thread::spawn(move || $read_abs(subscriber, IPC_ADDR))
                    .join()
                    .unwrap();
            }
        };

        (
            async =>
            func_name = $func_name:ident,
            publisher = $publisher:ident,
            subscriber = $subscriber:ident,
            read_abs = $read_abs:ident,
        ) => {
            #[tokio::test(flavor = "multi_thread")]
            async fn $func_name() {
                const IPC_ADDR: &str = concat!("ipc:///tmp/libstock_", stringify!($read_abs), stringify!($subscriber), ".ipc");
                let publisher = $publisher::new().expect("failed to create publisher");
                let subscriber = $subscriber::new().expect("failed to create subscriber");

                let publisher_thread =
                    tokio::task::spawn(abstract_async_write_function(publisher, IPC_ADDR));
                tokio::task::spawn($read_abs(subscriber, IPC_ADDR))
                    .await
                    .unwrap();

                publisher_thread.abort();
            }
        };
    }

    mod nanomsg {
        use super::super::nanomsg::{NanomsgPublisher, NanomsgSubscriber};
        use super::*;

        build_test!(
            sync =>
            func_name = sync_read,
            publisher = NanomsgPublisher,
            subscriber = NanomsgSubscriber,
            read_abs = abstract_read_function,
        );

        build_test!(
            sync =>
            func_name = sync_iter,
            publisher = NanomsgPublisher,
            subscriber = NanomsgSubscriber,
            read_abs = abstract_iter_function,
        );

        build_test!(
            async =>
            func_name = async_read,
            publisher = NanomsgPublisher,
            subscriber = NanomsgSubscriber,
            read_abs = abstract_async_read_function,
        );

        build_test!(
            async =>
            func_name = async_iter,
            publisher = NanomsgPublisher,
            subscriber = NanomsgSubscriber,
            read_abs = abstract_async_stream_function,
        );
    }

    mod zeromq {
        use super::super::zeromq::{ZeromqPublisher, ZeromqSubscriber};
        use super::*;

        build_test!(
            sync =>
            func_name = sync_read,
            publisher = ZeromqPublisher,
            subscriber = ZeromqSubscriber,
            read_abs = abstract_read_function,
        );

        build_test!(
            sync =>
            func_name = sync_iter,
            publisher = ZeromqPublisher,
            subscriber = ZeromqSubscriber,
            read_abs = abstract_iter_function,
        );

        build_test!(
            async =>
            func_name = async_read,
            publisher = ZeromqPublisher,
            subscriber = ZeromqSubscriber,
            read_abs = abstract_async_read_function,
        );

        build_test!(
            async =>
            func_name = async_iter,
            publisher = ZeromqPublisher,
            subscriber = ZeromqSubscriber,
            read_abs = abstract_async_stream_function,
        );
    }
}
