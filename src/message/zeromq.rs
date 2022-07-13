//! A basic encap methods of [`zmq`].

use tokio::io::{self, AsyncRead, AsyncWrite};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};

pub use zeromq::{PubSocket, SubSocket, ZmqError, ZmqResult};

use std::ops::{Deref, DerefMut};
use std::task::Poll;

use super::AsyncSubscribe;

/// A basic encap of [`zmq::Context`] for subscribing and publishing.
pub struct Zeromq<T> {
    socket: T,
}

impl<T> Zeromq<T> {
    pub async fn new_publish(path: &str) -> ZmqResult<Zeromq<PubSocket>> {
        let mut pub_socket = Zeromq {
            socket: PubSocket::new(),
        };
        pub_socket.bind(path).await?;
        // 空消息
        pub_socket.send(ZmqMessage::from("")).await?;
        Ok(pub_socket)
    }

    pub async fn new_subscribe(path: &str) -> ZmqResult<Zeromq<SubSocket>> {
        let mut sub_socket = Zeromq {
            socket: SubSocket::new(),
        };
        sub_socket.connect(path).await?;
        Ok(sub_socket)
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

impl AsyncRead for Zeromq<SubSocket> {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let mut response = self.socket.recv();

        let data = response.as_mut().poll(cx);

        match data {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))
            }
            Poll::Ready(Ok(data)) => {
                let data: Result<Vec<u8>, _> = data.try_into();

                match data {
                    Ok(data) => {
                        buf.put_slice(&data);
                        Poll::Ready(Ok(()))
                    }
                    Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e.to_string()))),
                }
            }
        }
    }
}

impl AsyncWrite for Zeromq<PubSocket> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        let buf = buf.to_vec();
        let buf_len = buf.len();
        let mut task = self.socket.send(ZmqMessage::from(buf));

        match task.as_mut().poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => {
                Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e.to_string())))
            }
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf_len)),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        unimplemented!("Zeromq does not support flush.")
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        unimplemented!("Shutdown with `self.socket.close()`.")
    }
}

#[async_trait::async_trait]
impl AsyncSubscribe for Zeromq<SubSocket> {
    type Result = ZmqResult<()>;

    async fn subscribe(&mut self, topic: &str) -> Self::Result {
        self.socket.subscribe(topic).await
    }
}
