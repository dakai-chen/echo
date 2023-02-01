mod boxed;
mod boxed_local;
mod buf_list;
mod collect;
mod ext;
mod limited;
mod map_err;
mod next;
mod stream;

pub use boxed::BoxBody;
pub use boxed_local::LocalBoxBody;
pub use collect::{Collect, Collected};
pub use ext::BodyExt;
pub use limited::{DataTooLarge, Limited};
pub use map_err::MapErr;
pub use next::Next;
pub use stream::{BodyStream, StreamBody};

use std::borrow::Cow;
use std::convert::Infallible;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use bytes::{Buf, Bytes};
pub use http_body::{Frame, SizeHint};

pub trait Body {
    type Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>>;

    fn size_hint(&self) -> SizeHint {
        SizeHint::default()
    }
}

impl<B> Body for &mut B
where
    B: Body + Unpin + ?Sized,
{
    type Error = B::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Pin::new(&mut **self).poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        (**self).size_hint()
    }
}

impl<P> Body for Pin<P>
where
    P: Unpin + DerefMut,
    P::Target: Body,
{
    type Error = <P::Target as Body>::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.get_mut().as_mut().poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        (**self).size_hint()
    }
}

impl<B> Body for Box<B>
where
    B: Body + Unpin + ?Sized,
{
    type Error = B::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Pin::new(&mut **self).poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        (**self).size_hint()
    }
}

impl Body for () {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        Poll::Ready(None)
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(0)
    }
}

impl Body for &'static [u8] {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Frame::data(Bytes::from_static(std::mem::take(
                self.get_mut(),
            ))))))
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for Vec<u8> {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Frame::data(Bytes::from(std::mem::take(
                self.get_mut(),
            ))))))
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for Cow<'static, [u8]> {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        match self.get_mut() {
            Cow::Borrowed(v) => Pin::new(v).poll_frame(cx),
            Cow::Owned(v) => Pin::new(v).poll_frame(cx),
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for &'static str {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Frame::data(Bytes::from_static(
                std::mem::take(self.get_mut()).as_bytes(),
            )))))
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for String {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Frame::data(Bytes::from(std::mem::take(
                self.get_mut(),
            ))))))
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for Cow<'static, str> {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        match self.get_mut() {
            Cow::Borrowed(v) => Pin::new(v).poll_frame(cx),
            Cow::Owned(v) => Pin::new(v).poll_frame(cx),
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}

impl Body for Bytes {
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        if self.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(Frame::data(std::mem::take(self.get_mut())))))
        }
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.len() as u64)
    }
}
