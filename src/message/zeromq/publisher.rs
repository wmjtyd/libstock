use std::task::Poll;

use super::super::traits::{AsyncWrite, Write};

use crate::message::{traits::{Publisher, Bind, SyncPublisher, AsyncPublisher}, MessageError};

use super::{common::construct_zeromq, ZeromqError};

construct_zeromq!(
    name = ZeromqPublisher,
    socket_type = zmq2::SocketType::PUB,
    category = Publisher
);

impl Bind for ZeromqPublisher {
    type Err = MessageError;

    fn bind(&mut self, uri: &str) -> Result<(), Self::Err> {
        self.socket.bind(uri).map_err(ZeromqError::BindFailed)?;

        Ok(())
    }

    fn unbind(&mut self, uri: &str) -> Result<(), Self::Err> {
        self.socket.unbind(uri).map_err(ZeromqError::UnbindFailed)?;

        Ok(())
    }
}

impl Write for ZeromqPublisher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let response = self.socket.send(buf, 0);

        match response {
            Ok(()) => Ok(buf.len()),
            Err(e) => Err(e.into())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // Zeromq doesn't need flush.
        Ok(())
    }
}

impl AsyncWrite for ZeromqPublisher {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let result = self.socket.send(buf, zmq2::DONTWAIT);

        match result {
            Ok(_) => {
                Poll::Ready(Ok(buf.len()))
            }
            Err(zmq2::Error::EAGAIN) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        // Zeromq doesn't need flush.
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        // We don't need to close anything ;)
        Poll::Ready(Ok(()))
    }
}

impl SyncPublisher for ZeromqPublisher {}

impl AsyncPublisher for ZeromqPublisher {}
