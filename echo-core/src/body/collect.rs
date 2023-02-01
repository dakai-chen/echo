use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::HeaderMap;

use super::buf_list::BufList;
use super::{Body, Buf, Bytes, Frame};

pin_project_lite::pin_project! {
    pub struct Collect<B> {
        collected: Option<Collected>,
        #[pin]
        body: B,
    }
}

impl<B> Collect<B> {
    #[inline]
    pub fn new(body: B) -> Self {
        Self {
            collected: Some(Collected::default()),
            body,
        }
    }
}

impl<B: Body> Future for Collect<B> {
    type Output = Result<Collected, B::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let frame = futures_core::ready!(this.body.as_mut().poll_frame(cx));

            let frame = if let Some(frame) = frame {
                frame?
            } else {
                return Poll::Ready(Ok(this
                    .collected
                    .take()
                    .expect("future must not be polled after it returned `Poll::Ready`")));
            };

            this.collected.as_mut().unwrap().push_frame(frame)
        }
    }
}

#[derive(Debug)]
pub struct Collected {
    bufs: BufList<Bytes>,
    trailers: Option<HeaderMap>,
}

impl Collected {
    pub fn aggregate(self) -> impl Buf {
        self.bufs
    }

    pub fn to_bytes(mut self) -> Bytes {
        self.bufs.copy_to_bytes(self.bufs.remaining())
    }

    pub fn trailers(&self) -> Option<&HeaderMap> {
        self.trailers.as_ref()
    }

    fn push_frame(&mut self, frame: Frame<Bytes>) {
        let frame = match frame.into_data() {
            Ok(data) => {
                self.bufs.push(data);
                return;
            }
            Err(frame) => frame,
        };

        if let Ok(trailers) = frame.into_trailers() {
            if let Some(cur) = &mut self.trailers {
                cur.extend(trailers.into_iter());
            } else {
                self.trailers = Some(trailers);
            }
        }
    }
}

impl Body for Collected {
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let frame = if let Some(data) = self.bufs.pop() {
            Frame::data(data)
        } else if let Some(trailers) = self.trailers.take() {
            Frame::trailers(trailers)
        } else {
            return Poll::Ready(None);
        };
        Poll::Ready(Some(Ok(frame)))
    }
}

impl Default for Collected {
    fn default() -> Self {
        Self {
            bufs: BufList::default(),
            trailers: None,
        }
    }
}
