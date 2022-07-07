//! A basic encap methods of [`zmq`].

use zmq::{Context, Socket};

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

pub use zmq::Error as MessageError;
pub use zmq::Result as MessageResult;
pub use zmq::SocketType;

/// A basic encap of [`zmq::Context`] for subscribing and publishing.
pub struct Zeromq {
    socket: Socket,
}

impl Zeromq {
    /// Construct a [`Zeromq`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::zeromq::{Zeromq, ZeromqProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_zeromq = Zeromq::new_publish("tcp://127.0.0.1:5432");
    /// let pub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", ZeromqProtocol::PUB);
    /// ```
    pub fn new(uri: &str, socket_type: SocketType) -> MessageResult<Self> {
        use SocketType::{PUB, SUB};

        let context = Context::new();
        let socket = context.socket(socket_type)?;

        match socket_type {
            PUB => {
                socket.bind(uri)?;
            }
            SUB => {
                socket.connect(uri)?;
            }
            _ => unimplemented!(),
        }

        Ok(Zeromq { socket })
    }

    /// Construct a publishable [`Zeromq`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::zeromq::{Zeromq, ZeromqProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_zeromq = Zeromq::new_publish("tcp://127.0.0.1:5432");
    /// let pub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", ZeromqProtocol::PUB);
    /// ```
    pub fn new_publish(path: &str) -> MessageResult<Self> {
        Self::new(path, SocketType::PUB)
    }

    /// Construct a subscribable [`Zeromq`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::zeromq::{Zeromq, ZeromqProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let sub_zeromq = Zeromq::new_subscribe("tcp://127.0.0.1:5432");
    /// let sub_zeromq = Zeromq::new("tcp://127.0.0.1:5432", ZeromqProtocol::SUB);
    /// ```
    pub fn new_subscribe(path: &str) -> MessageResult<Self> {
        Self::new(path, SocketType::SUB)
    }
}

impl Deref for Zeromq {
    type Target = Socket;

    /// Get the underlying [`Socket`] instance.
    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for Zeromq {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}

impl Read for Zeromq {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let response = self.socket.recv_bytes(0);

        match response {
            Ok(n) => {
                buf.copy_from_slice(&n);
                Ok(n.len())
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl Write for Zeromq {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let response = self.socket.send(buf, 0);

        match response {
            Ok(()) => Ok(buf.len()),
            Err(e) => Err(e.into()),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!("Zeromq does not support flush.")
    }
}

#[cfg(test)]
mod tests {
    #[cfg(with_zeromq_test)]
    #[test]
    fn test_zeromq_pub_sub() {
        use crate::message::zeromq::Zeromq;
        use std::io::{Read, Write};

        const LISTEN_ADDRESS: &str = "tcp://127.0.0.1:10352";

        let mut pub_zeromq =
            Zeromq::new_publish(LISTEN_ADDRESS).expect("failed to create Zeromq publish socket");
        let mut sub_zeromq = Zeromq::new_subscribe(LISTEN_ADDRESS)
            .expect("failed to create Zeromq subscribe socket");

        sub_zeromq
            .set_subscribe(b"TEST")
            .expect("failed to set subscribe");

        let content = b"TEST Hello, World!";
        let mut recv_buf = [0u8; 18];

        pub_zeromq
            .write_all(content)
            .expect("failed to write to pub_zeromq");
        let written = sub_zeromq
            .read(&mut recv_buf)
            .expect("failed to read from sub_zeromq");

        assert_eq!(content, &recv_buf);
        assert_eq!(written, content.len());
    }
}
