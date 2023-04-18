//use futures::FutureExt;
use pin_project::pin_project;
use std::future::Future;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::{
    io::{AsyncRead, ReadBuf},
    time::{Duration, Instant, Sleep},
};

#[pin_project]
pub struct SlowRead<R> {
    #[pin]
    pub reader: R,
    #[pin]
    pub sleep: Sleep,
}

impl<R> SlowRead<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            sleep: tokio::time::sleep(Default::default()),
        }
    }
}

impl<R> AsyncRead for SlowRead<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut this = self.project();

        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.sleep.reset(Instant::now() + Duration::from_millis(25));
                this.reader.poll_read(cx, buf)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
