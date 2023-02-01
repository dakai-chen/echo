use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct Map<S, F> {
    service: S,
    f: F,
}

impl<S, F> Map<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res, Err> Service<Req> for Map<S, F>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Result<Res, Err>,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = MapFuture<'f, S, F, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        MapFuture {
            slf: self,
            fut: self.service.call(req),
        }
    }
}

impl<S, F> fmt::Debug for Map<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct MapFuture<'f, S, F, Req>
    where
        S: Service<Req>,
    {
        slf: &'f Map<S, F>,
        #[pin]
        fut: S::Future<'f>,
    }
}

impl<'f, S, F, Req, Res, Err> Future for MapFuture<'f, S, F, Req>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Result<Res, Err>,
{
    type Output = Result<Res, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Ready(res) => Poll::Ready((this.slf.f)(res)),
            Poll::Pending => Poll::Pending,
        }
    }
}
