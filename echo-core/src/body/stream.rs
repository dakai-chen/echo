use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;

use super::{Body, Bytes, Frame, SizeHint};

pin_project_lite::pin_project! {
    #[derive(Debug, Clone, Copy)]
    pub struct StreamBody<S> {
        #[pin]
        stream: S,
    }
}

impl<S> StreamBody<S> {
    #[inline]
    pub fn new(stream: S) -> Self {
        Self { stream }
    }

    #[inline]
    pub fn get_ref(&self) -> &S {
        &self.stream
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    #[inline]
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut S> {
        self.project().stream
    }

    #[inline]
    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S, E> Body for StreamBody<S>
where
    S: Stream<Item = Result<Frame<Bytes>, E>>,
{
    type Error = E;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.project().stream.poll_next(cx)
    }
}

impl<S: Stream> Stream for StreamBody<S> {
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().stream.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.stream.size_hint()
    }
}

pin_project_lite::pin_project! {
    #[derive(Debug, Clone, Copy)]
    pub struct BodyStream<B> {
        #[pin]
        body: B,
    }
}

impl<B> BodyStream<B> {
    #[inline]
    pub fn new(body: B) -> Self {
        Self { body }
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

impl<B: Body> Stream for BodyStream<B> {
    type Item = Result<Frame<Bytes>, B::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.as_mut().project().body.poll_frame(cx)
    }
}

impl<B: Body> Body for BodyStream<B> {
    type Error = B::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.project().body.poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}
