use std::task::Poll;

use crate::message::traits::{Publisher, Bind, SyncPublisher, AsyncPublisher, Write, AsyncWrite};

use super::{common::construct_nanomsg, NanomsgError};

construct_nanomsg!(
    name = NanomsgPublisher,
    socket_type = nanomsg::Protocol::Sub,
    category = Publisher
);

impl Bind for NanomsgPublisher {
    type Err = NanomsgError;

    fn bind(&mut self, uri: &str) -> Result<(), Self::Err> {
        let endpoint = self.socket.bind(uri).map_err(NanomsgError::BindFailed)?;

        self.endpoint.insert(uri.to_string(), endpoint);
        Ok(())
    }

    fn unbind(&mut self, uri: &str) -> Result<(), Self::Err> {
        // Try to remove the “uri” from endpoint; if none is removed,
        // we considers it not existed.
        let endp = self.endpoint.remove(uri);
        
        if let Some(mut endp) = endp {
            Ok(endp.shutdown().map_err(NanomsgError::UnbindFailed)?)
        } else {
            Err(NanomsgError::NoSuchEndpoint(uri.to_string()))?
        }
    }
}

impl Write for NanomsgPublisher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.socket.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.socket.flush()
    }
}

impl SyncPublisher for NanomsgPublisher {}

impl AsyncWrite for NanomsgPublisher {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let result = self.socket.nb_write(buf);

        match result {
            Ok(len) => Poll::Ready(Ok(len)),
            Err(nanomsg::Error::TryAgain) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            },
            Err(err) => Poll::Ready(Err(err.into())),
        }
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), std::io::Error>> {
        // No needs to flush or shutdown. Just drop it :)
        Poll::Ready(Ok(()))
    }
}

impl AsyncPublisher for NanomsgPublisher {}
