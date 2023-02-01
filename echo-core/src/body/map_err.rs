use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{Body, Bytes, Frame, SizeHint};

pin_project_lite::pin_project! {
    #[derive(Clone, Copy)]
    pub struct MapErr<B, F> {
        #[pin]
        body: B,
        f: F
    }
}

impl<B, F> MapErr<B, F> {
    #[inline]
    pub fn new(body: B, f: F) -> Self {
        Self { body, f }
    }

    #[inline]
    pub fn get_ref(&self) -> &B {
        &self.body
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut B {
        &mut self.body
    }

    #[inline]
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut B> {
        self.project().body
    }

    #[inline]
    pub fn into_inner(self) -> B {
        self.body
    }
}

impl<B, F, E> Body for MapErr<B, F>
where
    B: Body,
    F: FnMut(B::Error) -> E,
{
    type Error = E;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.project();
        match this.body.poll_frame(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(Ok(frame))) => Poll::Ready(Some(Ok(frame))),
            Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err((this.f)(err)))),
        }
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}

impl<B, F> fmt::Debug for MapErr<B, F>
where
    B: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("body", &self.body)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
