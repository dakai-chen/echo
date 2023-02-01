use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct MapOk<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapOk<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Req, Res> Service<Req> for MapOk<S, F>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Res,
{
    type Response = Res;
    type Error = S::Error;
    type Future<'f> = MapOkFuture<'f, S, F, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        MapOkFuture {
            slf: self,
            fut: self.service.call(req),
        }
    }
}

impl<S, F> fmt::Debug for MapOk<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapOk")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct MapOkFuture<'f, S, F, Req>
    where
        S: Service<Req>,
    {
        slf: &'f MapOk<S, F>,
        #[pin]
        fut: S::Future<'f>,
    }
}

impl<'f, S, F, Req, Res> Future for MapOkFuture<'f, S, F, Req>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Res,
{
    type Output = Result<Res, S::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.fut.poll(cx) {
            Poll::Ready(Ok(res)) => Poll::Ready(Ok((this.slf.f)(res))),
            Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
            Poll::Pending => Poll::Pending,
        }
    }
}
