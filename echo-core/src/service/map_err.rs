use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct MapErr<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapErr<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Err> Service<Req> for MapErr<S, F>
where
    S: Service<Req>,
    F: Fn(S::Error) -> Err,
{
    type Response = S::Response;
    type Error = Err;
    type Future<'f> = MapErrFuture<'f, S, F, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        MapErrFuture {
            slf: self,
            fut: self.service.call(req),
        }
    }
}

impl<S, F> fmt::Debug for MapErr<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapErr")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct MapErrFuture<'f, S, F, Req>
    where
        S: Service<Req>,
    {
        slf: &'f MapErr<S, F>,
        #[pin]
        fut: S::Future<'f>,
    }
}

impl<'f, S, F, Req, Err> Future for MapErrFuture<'f, S, F, Req>
where
    S: Service<Req>,
    F: Fn(S::Error) -> Err,
{
    type Output = Result<S::Response, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Ready(Ok(res)) => Poll::Ready(Ok(res)),
            Poll::Ready(Err(err)) => Poll::Ready(Err((this.slf.f)(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}
