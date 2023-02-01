use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use echo_core::BoxError;

pin_project_lite::pin_project! {
    #[project = RouteFutureProj]
    pub enum RouteFuture<Fut> {
        A { #[pin] fut: Fut },
        B { err: Option<BoxError> },
    }
}

impl<Fut, Res, Err> Future for RouteFuture<Fut>
where
    Fut: Future<Output = Result<Res, Err>>,
    Err: Into<BoxError>,
{
    type Output = Result<Res, BoxError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            RouteFutureProj::A { fut } => fut.poll(cx).map_err(Into::into),
            RouteFutureProj::B { err } => Poll::Ready(Err(err
                .take()
                .expect("future must not be polled after it returned `Poll::Ready`"))),
        }
    }
}
