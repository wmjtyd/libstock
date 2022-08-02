//! A basic encap methods of [`zmq`].
//!
//! Note that:
//! - We return [`MessageError`] to maintain the
//!   same error type as the other implementations.

mod common;
mod publisher;
mod subscriber;

pub use subscriber::ZeromqSubscriber;

#[derive(thiserror::Error, Debug)]
pub enum ZeromqError {
    #[error("Unable to create socket: {0}")]
    CreateSocketFailed(zmq2::Error),

    #[error("Failed to connect to socket: {0}")]
    ConnectFailed(zmq2::Error),

    #[error("Failed to receive: {0}")]
    RecvFailed(zmq2::Error),

    #[error("Failed to send: {0}")]
    SendFailed(zmq2::Error),
}

pub type ZeromqResult<T> = Result<T, ZeromqError>;

// use super::traits::Connect;

// construct_zeromq!(
//     name = ZeromqSubscriber,
//     socket_type = SocketType::SUB,
// );

// impl Connect for ZeromqSubscriber {
//     type Err = MessageError;

//     fn connect(&mut self, uri: &str) -> Result<(), Self::Err> {
//         let socket = self.socket;

//         Ok(socket.connect(uri).map_err(ZeromqError::ConnectFailed)?)
//     }
// }

// impl Read for ZeromqSubscriber {
//     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
//         self.socket.recv_into(buf, 0).into()
//     }
// }

// impl AsyncRead for ZeromqSubscriber {
//     fn poll_read(
//         self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> std::task::Poll<std::io::Result<()>> {
//         let mut waker = cx.waker().wake();
//         tokio::task::spawn_blocking(|| {

//         });

//         self.socket.recv_msg(zmq::DONTWAIT)
//     }
// }

// impl Iterator for ZeromqSubscriber {
//     type Item = MessageResult<Message>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let response = self.socket.recv_msg(0);
//     }
// }

// // /// A basic encap of [`zmq::Context`] for subscribing and publishing.
// // pub struct Zeromq {
// //     context: Context,
// //     socket: Socket,
// // }

// // impl SocketConstructor for Zeromq {
// //     type Err = MessageError;

// //     type Subscriber;

// //     type Publisher;

// //     fn new_subscriber() -> Result<Self::Subscriber, Self::Err> {
// //         todo!()
// //     }

// //     fn new_publisher() -> Result<Self::Publisher, Self::Err> {
// //         todo!()
// //     }
// // }

// // impl Zeromq {
// //     /// Construct a [`Zeromq`] instance.
// //     ///
// //     /// # Example
// //     ///
// //     /// ```
// //     /// use wmjtyd_libstock::message::zeromq::{Zeromq, SocketType};
// //     ///
// //     /// // The both are equivalent: It will create a publishable Nanomsg socket.
// //     /// let pub_zeromq = Zeromq::new_publish("tcp://127.0.0.1:5432");
// //     /// let pub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", SocketType::PUB);
// //     /// ```
// //     pub fn new(uri: &str, socket_type: SocketType) -> MessageResult<Self> {
// //         use SocketType::{PUB, SUB};

// //         let context = Context::new();
// //         let socket = context.socket(socket_type)?;

// //         match socket_type {
// //             PUB => {
// //                 socket.bind(uri)?;
// //             }
// //             SUB => {
// //                 socket.connect(uri)?;
// //             }
// //             _ => unimplemented!(),
// //         }

// //         Ok(Zeromq { socket })
// //     }

// //     /// Construct a publishable [`Zeromq`] instance.
// //     ///
// //     /// # Example
// //     ///
// //     /// ```
// //     /// use wmjtyd_libstock::message::zeromq::{Zeromq, SocketType};
// //     ///
// //     /// // The both are equivalent: It will create a publishable Nanomsg socket.
// //     /// let pub_zeromq = Zeromq::new_publish("tcp://127.0.0.1:5432");
// //     /// let pub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", SocketType::PUB);
// //     /// ```
// //     pub fn new_publish(path: &str) -> MessageResult<Self> {
// //         Self::new(path, SocketType::PUB)
// //     }

// //     /// Construct a subscribable [`Zeromq`] instance.
// //     ///
// //     /// # Example
// //     ///
// //     /// ```
// //     /// use wmjtyd_libstock::message::zeromq::{Zeromq, SocketType};
// //     ///
// //     /// // The both are equivalent: It will create a publishable Nanomsg socket.
// //     /// let sub_zeromq = Zeromq::new_subscribe("tcp://127.0.0.1:5432");
// //     /// let sub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", SocketType::SUB);
// //     /// ```
// //     pub fn new_subscribe(path: &str) -> MessageResult<Self> {
// //         Self::new(path, SocketType::SUB)
// //     }
// // }

// // impl Deref for Zeromq {
// //     type Target = Socket;

// //     /// Get the underlying [`Socket`] instance.
// //     fn deref(&self) -> &Self::Target {
// //         &self.socket
// //     }
// // }

// // impl DerefMut for Zeromq {
// //     fn deref_mut(&mut self) -> &mut Self::Target {
// //         &mut self.socket
// //     }
// // }

// // impl Read for Zeromq {
// //     fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
// //         let response = self.socket.recv_bytes(0);

// //         match response {
// //             Ok(n) => {
// //                 buf.copy_from_slice(&n);
// //                 Ok(n.len())
// //             }
// //             Err(e) => Err(e.into()),
// //         }
// //     }
// // }

// // impl Write for Zeromq {
// //     fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
// //         let response = self.socket.send(buf, 0);

// //         match response {
// //             Ok(()) => Ok(buf.len()),
// //             Err(e) => Err(e.into()),
// //         }
// //     }

// //     fn flush(&mut self) -> std::io::Result<()> {
// //         unimplemented!("Zeromq does not support flush.")
// //     }
// // }

// // impl Subscribe for Zeromq {
// //     type Result = MessageResult<()>;

// //     fn subscribe(&mut self, topic: &str) -> Self::Result {
// //         self.socket.set_subscribe(topic.as_bytes())
// //     }
// // }
