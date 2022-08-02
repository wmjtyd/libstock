use std::mem::MaybeUninit;
use std::task::Poll;

use super::common::construct_nanomsg;
use super::NanomsgError;
use crate::message::traits::{
    AsyncRead,
    AsyncSubscriber,
    Connect,
    Read,
    Stream,
    SubscribeStreamItem,
    Subscriber,
    SyncSubscriber,
};
use crate::message::MessageError;

construct_nanomsg!(
    name = NanomsgSubscriber,
    socket_type = nanomsg::Protocol::Sub,
    category = Subscriber
);

impl Connect for NanomsgSubscriber {
    type Err = MessageError;

    fn connect(&mut self, uri: &str) -> Result<(), Self::Err> {
        let endpoint = self
            .socket
            .connect(uri)
            .map_err(NanomsgError::ConnectFailed)?;

        self.endpoint.insert(uri.to_string(), endpoint);
        Ok(())
    }

    fn disconnect(&mut self, uri: &str) -> Result<(), Self::Err> {
        // Try to remove the “uri” from endpoint; if none is removed,
        // we considers it not existed.
        let endp = self.endpoint.remove(uri);

        if let Some(mut endp) = endp {
            Ok(endp.shutdown().map_err(NanomsgError::DisconnectFailed)?)
        } else {
            Err(NanomsgError::NoSuchEndpoint(uri.to_string()))?
        }
    }
}

impl Read for NanomsgSubscriber {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.read(buf)
    }
}

impl Iterator for NanomsgSubscriber {
    type Item = SubscribeStreamItem<<Self as SyncSubscriber>::Err>;

    fn next(&mut self) -> Option<Self::Item> {
        // The buffer can place at most 4096 bytes (4KB) of data.
        // It is enough for our finance data.
        let mut buf = {
            // SAFETY: The array will be initiated when reading to it.
            // Besides, [u8; 4096] should not be a big deal.
            #[allow(clippy::uninit_assumed_init)]
            unsafe {
                std::mem::MaybeUninit::<[u8; 4096]>::uninit().assume_init()
            }
        };
        let result = self.read(&mut buf).map_err(NanomsgError::ReadFailed);

        match result {
            Ok(len) => {
                let owned_data = buf[..len].to_vec();
                Some(Ok(owned_data))
            }
            Err(e) => Some(Err(e.into())),
        }
    }
}

impl SyncSubscriber for NanomsgSubscriber {
    type Err = MessageError;
}

impl AsyncRead for NanomsgSubscriber {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let previously_filled_len = buf.filled().len();
        let not_filled_buf = buf.initialize_unfilled();
        // Non-blocking read!
        let result = self.socket.nb_read(not_filled_buf);

        match result {
            Ok(len) => {
                // Set the currented “filled” length to
                // the sum of the previously filled len
                // and the written bytes.
                buf.set_filled(previously_filled_len + len);

                Poll::Ready(Ok(()))
            }
            Err(nanomsg::Error::TryAgain) => {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e.into())),
        }
    }
}

impl Stream for NanomsgSubscriber {
    type Item = SubscribeStreamItem<<Self as AsyncSubscriber>::Err>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        // The buffer can place at most 4096 bytes (4KB) of data.
        // It is enough for our finance data.
        let mut buf = create_uninitiated_array::<u8, 4096>();
        let mut tokio_read_buf = tokio::io::ReadBuf::uninit(&mut buf);

        let read_result = self
            .poll_read(cx, &mut tokio_read_buf)
            .map_err(NanomsgError::ReadFailed);

        match read_result {
            Poll::Ready(Ok(())) => {
                let filled_buf = tokio_read_buf.filled().to_vec();
                Poll::Ready(Some(Ok(filled_buf)))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e.into()))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncSubscriber for NanomsgSubscriber {
    type Err = MessageError;
}

/// Copy from https://doc.rust-lang.org/nightly/core/mem/union.MaybeUninit.html#method.uninit_array.
/// We use it because of it is not stablized in Rust 1.62 yet 😞
fn create_uninitiated_array<T, const N: usize>() -> [MaybeUninit<T>; N] {
    // SAFETY: An uninitialized `[MaybeUninit<_>; LEN]` is valid.
    unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() }
}
