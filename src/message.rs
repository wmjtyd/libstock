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
