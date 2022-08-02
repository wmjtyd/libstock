use std::task::Poll;

use crate::message::traits::{AsyncRead, Stream};
use crate::message::MessageResult;

use crate::message::zeromq::ZeromqError;
use crate::message::{traits::AsyncSubscriber, MessageError};

use crate::message::traits::Read;

use crate::message::traits::{Connect, Subscriber, SyncSubscriber};

use super::common::construct_zeromq;

construct_zeromq!(
    name = ZeromqSubscriber,
    socket_type = zmq::SocketType::SUB,
    category = Subscriber
);

impl Connect for ZeromqSubscriber {
    type Err = MessageError;

    fn connect(&mut self, uri: &str) -> Result<(), Self::Err> {
        self.socket
            .connect(uri)
            .map_err(ZeromqError::ConnectFailed)?;

        Ok(())
    }
}

impl Read for ZeromqSubscriber {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv_into(buf, 0).map_err(|e| e.into())
    }
}

impl Iterator for ZeromqSubscriber {
    type Item = MessageResult<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        let data = self.socket.recv_bytes(0).map_err(ZeromqError::RecvFailed);

        match data {
            Ok(buf) => Some(Ok(buf)),
            Err(e) => Some(Err(e.into())),
        }
    }
}

impl AsyncRead for ZeromqSubscriber {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let result = self.socket.recv_msg(zmq::DONTWAIT);

        match result {
            Ok(m) => {
                buf.put_slice(&m);
                Poll::Ready(Ok(()))
            }
            Err(zmq::Error::EAGAIN) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }
}

impl Stream for ZeromqSubscriber {
    type Item = MessageResult<Vec<u8>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let result = self.socket.recv_bytes(zmq::DONTWAIT);

        match result {
            Ok(m) => Poll::Ready(Some(Ok(m))),
            Err(zmq::Error::EAGAIN) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Some(Err(ZeromqError::RecvFailed(e).into()))),
        }
    }
}

impl SyncSubscriber for ZeromqSubscriber {
    type Err = MessageError;
}

impl AsyncSubscriber for ZeromqSubscriber {
    type Err = MessageError;
}
