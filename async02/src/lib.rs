#![allow(clippy::uninit_vec)]
use async_std::io::ReadExt;
use async_trait::async_trait;
use futures::io::AsyncRead;
use futures::Future;
use pin_project::pin_project;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

pub struct TracingReader<R>
where
    R: AsyncRead,
{
    pub inner: R,
}

#[async_trait]
pub trait SimpleRead {
    async fn simple_read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

#[async_trait]
impl<R> SimpleRead for TracingReader<R>
where
    R: AsyncRead + Send + Unpin,
{
    async fn simple_read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use futures_timer::Delay;
        use std::time::Duration;

        // artificial slowdown
        tracing::debug!("doing delay...");
        Delay::new(Duration::from_millis(50)).await;
        tracing::debug!("doing delay...done!");

        // reading
        tracing::debug!("doing read...");
        let res = self.inner.read(buf).await;
        tracing::debug!("doing read...done!");
        res
    }
}

#[pin_project]
pub struct SimpleAsyncReader<R>
where
    R: SimpleRead,
{
    pub state: State<R>,
}

type BoxFut<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub enum State<R> {
    Idle(R, Vec<u8>),
    Pending(BoxFut<(R, Vec<u8>, io::Result<usize>)>),
    Transitional,
}

impl<R> AsyncRead for SimpleAsyncReader<R>
where
    // new: R must now be `'static`, since it's captured
    // by the future which is, itself, `'static`.
    R: SimpleRead + Send + 'static,
{
    #[tracing::instrument(skip(self, buf))]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let proj = self.project();
        let mut state = State::Transitional;
        std::mem::swap(proj.state, &mut state);

        let mut fut = match state {
            State::Idle(mut inner, mut internal_buf) => {
                tracing::debug!("getting new future...");
                internal_buf.clear();
                internal_buf.reserve(buf.len());
                unsafe { internal_buf.set_len(buf.len()) }

                Box::pin(async move {
                    let res = inner.simple_read(&mut internal_buf[..]).await;
                    (inner, internal_buf, res)
                })
            }
            State::Pending(fut) => {
                tracing::debug!("polling existing future...");
                fut
            }
            State::Transitional => unreachable!(),
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready((inner, mut internal_buf, result)) => {
                tracing::debug!("future was ready!");
                if let Ok(n) = &result {
                    let n = *n;
                    unsafe { internal_buf.set_len(n) }

                    let dst = &mut buf[..n];
                    let src = &internal_buf[..];
                    dst.copy_from_slice(src);
                } else {
                    unsafe { internal_buf.set_len(0) }
                }
                *proj.state = State::Idle(inner, internal_buf);
                Poll::Ready(result)
            }
            Poll::Pending => {
                tracing::debug!("future was pending!");
                *proj.state = State::Pending(fut);
                Poll::Pending
            }
        }
    }
}
