//! A basic encap methods of [`zmq`].

use zeromq::{SubSocket, PubSocket, Socket, SocketSend, ZmqMessage, SocketRecv};

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use super::Subscribe;

type MessageResult<T> = Result<T, String>;

/// A basic encap of [`zmq::Context`] for subscribing and publishing.
pub struct Zeromq<T> {
    socket: T,
}

impl<T> Zeromq<T> {

    pub fn new_publish(path: &str) -> Zeromq<PubSocket> {
        let pub_socket = Zeromq {socket: PubSocket::new()};
        pub_socket.bind(path);
        // 空消息
        pub_socket.send(ZmqMessage::from(""));
        pub_socket
    }

    pub fn new_subscribe(path: &str) -> Zeromq<SubSocket> {
        let sub_socket = Zeromq {socket: SubSocket::new()};
        sub_socket.connect(path);
        sub_socket
    }
}

impl<T> Deref for Zeromq<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl<T> DerefMut for Zeromq<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}

impl Read for Zeromq<SubSocket> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
         async_std::task::block_on(self.socket.recv())
    }
}

impl Write for Zeromq<PubSocket> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.send(message)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        unimplemented!("Zeromq does not support flush.")
    }
}

impl Subscribe for Zeromq<SubSocket> {
    type Result = Result<(), ZmqError>;

    fn subscribe(&mut self, topic: &str) -> Self::Result {
        self.socket.subscribe(topic).await
    }
}