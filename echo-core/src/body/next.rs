use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::{Body, Bytes, Frame};

#[must_use = "futures don't do anything unless polled"]
#[derive(Debug)]
pub struct Next<'a, B: ?Sized>(pub(crate) &'a mut B);

impl<'a, B: Body + Unpin + ?Sized> Future for Next<'a, B> {
    type Output = Option<Result<Frame<Bytes>, B::Error>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll_frame(cx)
    }
}
