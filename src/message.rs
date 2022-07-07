//! A basic encap methods of [`nanomsg`].

use nanomsg::Socket;

use std::io::{Read, Write};
use std::ops::Deref;

pub use nanomsg::Error as MessageError;
pub use nanomsg::Protocol as NanomsgProtocol;
pub use nanomsg::Result as MessageResult;

/// A basic encap of [`Socket`] for subscribing and publishing.
///
/// It will automatically bind the created socket to the specified path.
///
/// # Example
///
/// ```
/// use std::io::Write;
/// use wmjtyd_libstock::message::{Nanomsg, NanomsgProtocol};
///
/// let nanomsg = Nanomsg::new("/tmp/test.socket", NanomsgProtocol::Pub);
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
    /// use wmjtyd_libstock::message::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_nanomsg = Nanomsg::new("/tmp/test.socket", NanomsgProtocol::Pub);
    /// let pub_nanomsg = Nanomsg::new_publish("/tmp/test.socket");
    ///
    /// // The both are equivalent: It will create a subscribable Nanomsg socket.
    /// let sub_nanomsg = Nanomsg::new("/tmp/test.socket", NanomsgProtocol::Sub);
    /// let sub_nanomsg = Nanomsg::new_subscribe("/tmp/test.socket");
    /// ```
    pub fn new(path: &str, protocol: NanomsgProtocol) -> MessageResult<Self> {
        use NanomsgProtocol::{Pub, Sub};

        let socket = match protocol {
            Sub => {
                let mut s = Socket::new(protocol)?;
                s.connect(path)?;
                s
            },
            Pub => {
                let mut s = Socket::new(protocol)?;
                s.bind(path)?;
                s
            },
            _ => panic!("unsupported protocol"),
        };

        Ok(Self { socket })
    }

    /// Construct a publishable [`Nanomsg`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a publishable Nanomsg socket.
    /// let pub_nanomsg = Nanomsg::new_publish("/tmp/test.socket");
    /// let pub_nanomsg = Nanomsg::new("/tmp/test.socket", NanomsgProtocol::Pub);
    /// ```
    pub fn new_publish(path: &str) -> MessageResult<Self> {
        Self::new(path, NanomsgProtocol::Pub)
    }

    /// Construct a subscribable [`Nanomsg`] instance.
    ///
    /// # Example
    ///
    /// ```
    /// use wmjtyd_libstock::message::{Nanomsg, NanomsgProtocol};
    ///
    /// // The both are equivalent: It will create a subscribable Nanomsg socket.
    /// let sub_nanomsg = Nanomsg::new_subscribe("/tmp/test.socket");
    /// let sub_nanomsg = Nanomsg::new("/tmp/test.socket", NanomsgProtocol::Sub);
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
