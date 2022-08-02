//! A basic encap methods of [`nanomsg`].

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use nanomsg::Socket;
pub use nanomsg::{Error as MessageError, Protocol as NanomsgProtocol, Result as MessageResult};

/// A basic encap of [`Socket`] for subscribing and publishing.
///
/// It will automatically bind the created socket to the specified path.
///
/// # Example
///
/// ```
/// use std::io::Write;
/// use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};
///
/// let nanomsg = Nanomsg::new("ipc:///tmp/test.ipc", NanomsgProtocol::Pub);
///
/// if let Ok(mut nanomsg) = nanomsg {
///   nanomsg.write_all(b"Hello World!").ok();
/// }
/// ```
pub struct Nanomsg {
    socket: Socket,
}

impl Nanomsg {
    /// Construct a [`Nanomsg`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_nanomsg = Nanomsg::new("ipc:///tmp/test.ipc", NanomsgProtocol::Pub);
    /// let pub_nanomsg = Nanomsg::new_publish("ipc:///tmp/test.ipc");
    ///
    /// // The both are equivalent: It will create a subscribable Nanomsg socket.
    /// let sub_nanomsg = Nanomsg::new("ipc:///tmp/test.ipc", NanomsgProtocol::Sub);
    /// let sub_nanomsg = Nanomsg::new_subscribe("ipc:///tmp/test.ipc");
    /// ```
    pub fn new(path: &str, protocol: NanomsgProtocol) -> MessageResult<Self> {
        use NanomsgProtocol::{Pub, Sub};

        let socket = match protocol {
            Sub => {
                let mut s = Socket::new(protocol)?;
                s.connect(path)?;
                s
            }
            Pub => {
                let mut s = Socket::new(protocol)?;
                s.bind(path)?;
                s
            }
            _ => panic!("unsupported protocol"),
        };

        Ok(Self { socket })
    }

    /// Construct a publishable [`Nanomsg`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_nanomsg = Nanomsg::new_publish("ipc:///tmp/test.ipc");
    /// let pub_nanomsg = Nanomsg::new("ipc:///tmp/test.ipc", NanomsgProtocol::Pub);
    /// ```
    pub fn new_publish(path: &str) -> MessageResult<Self> {
        Self::new(path, NanomsgProtocol::Pub)
    }

    /// Construct a subscribable [`Nanomsg`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::nanomsg::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a subscribable Nanomsg socket.
    /// let sub_nanomsg = Nanomsg::new_subscribe("ipc:///tmp/test.ipc");
    /// let sub_nanomsg = Nanomsg::new("ipc:///tmp/test.ipc", NanomsgProtocol::Sub);
    /// ```
    pub fn new_subscribe(path: &str) -> MessageResult<Self> {
        Nanomsg::new(path, NanomsgProtocol::Sub)
    }
}

impl Deref for Nanomsg {
    type Target = Socket;

    /// Get the underlying [`Socket`] instance.
    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for Nanomsg {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}

impl Read for Nanomsg {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.read(buf)
    }
}

impl Write for Nanomsg {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }
}


impl Subscribe for Nanomsg {
    type Result = MessageResult<()>;

    fn subscribe(&mut self, topic: &str) -> Self::Result {
        self.socket.subscribe(topic.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    // skip: seems it will block the process.
    // to test it, pass `--cfg with_nanomsg_test`.
    #[cfg(with_nanomsg_test)]
    #[test]
    fn test_nanomsg_pub_sub() {
        use std::io::{Read, Write};

        const IPC_PATH: &str = "ipc:///tmp/libstock_test_nanomsg_pub_sub.ipc";
        let mut pub_nanomsg =
            super::Nanomsg::new_publish(IPC_PATH).expect("failed to create Nanomsg publish socket");
        let mut sub_nanomsg = super::Nanomsg::new_subscribe(IPC_PATH)
            .expect("failed to create Nanomsg subscribe socket");

        sub_nanomsg.subscribe("").expect("failed to subscribe ''");

        let content = b"Hello, World!";
        let mut recv_buf = [0u8; 13];

        pub_nanomsg
            .write_all(content)
            .expect("failed to write to pub_nanomsg");
        pub_nanomsg.flush().expect("failed to flush pub_nanomsg");
        let written = sub_nanomsg
            .read(&mut recv_buf)
            .expect("failed to read from sub_nanomsg");

        assert_eq!(content, &recv_buf);
        assert_eq!(written, content.len());
    }
}
