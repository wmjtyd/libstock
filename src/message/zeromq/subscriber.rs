use std::task::Poll;

use super::common::construct_zeromq;
use crate::message::traits::{
    AsyncRead,
    AsyncSubscriber,
    Connect,
    Read,
    Stream,
    SubscribeStreamItem,
    Subscriber,
    SyncSubscriber, Subscribe,
};
use crate::message::zeromq::ZeromqError;
use crate::message::MessageError;

construct_zeromq!(
    name = ZeromqSubscriber,
    socket_type = zmq2::SocketType::SUB,
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

    fn disconnect(&mut self, uri: &str) -> Result<(), Self::Err> {
        self.socket
            .disconnect(uri)
            .map_err(ZeromqError::DisconnectFailed)?;

        Ok(())
    }
}

impl Read for ZeromqSubscriber {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv_into(buf, 0).map_err(|e| e.into())
    }
}

impl Iterator for ZeromqSubscriber {
    type Item = SubscribeStreamItem<<Self as SyncSubscriber>::Err>;

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
        let result = self.socket.recv_msg(zmq2::DONTWAIT);

        match result {
            Ok(m) => {
                buf.put_slice(&m);
                Poll::Ready(Ok(()))
            }
            Err(zmq2::Error::EAGAIN) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }
}

impl Stream for ZeromqSubscriber {
    type Item = SubscribeStreamItem<<Self as AsyncSubscriber>::Err>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let result = self.socket.recv_bytes(zmq2::DONTWAIT);

        match result {
            Ok(m) => Poll::Ready(Some(Ok(m))),
            Err(zmq2::Error::EAGAIN) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Some(Err(ZeromqError::RecvFailed(e).into()))),
        }
    }
}

impl Subscribe for ZeromqSubscriber {
    type Err = MessageError;

    fn subscribe(&mut self, topic: &[u8]) -> Result<(), Self::Err> {
        Ok(self.socket.set_subscribe(topic).map_err(ZeromqError::SubscribeFailed)?)
    }

    fn unsubscribe(&mut self, topic: &[u8]) -> Result<(), Self::Err> {
        Ok(self.socket.set_unsubscribe(topic).map_err(ZeromqError::UnsubscribeFailed)?)
    }
}

impl SyncSubscriber for ZeromqSubscriber {
    type Err = MessageError;
}

impl AsyncSubscriber for ZeromqSubscriber {
    type Err = MessageError;
}
