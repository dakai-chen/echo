mod and_then;
mod boxed;
mod boxed_local;
mod ext;
mod map;
mod map_err;
mod map_ok;
mod map_request;
mod or_else;
mod service_fn;
mod then;

pub mod future {
    pub use super::and_then::AndThenFuture;
    pub use super::boxed::BoxFuture;
    pub use super::boxed_local::LocalBoxFuture;
    pub use super::map::MapFuture;
    pub use super::map_err::MapErrFuture;
    pub use super::map_ok::MapOkFuture;
    pub use super::or_else::OrElseFuture;
    pub use super::then::ThenFuture;
}

pub use and_then::AndThen;
pub use boxed::{ArcService, BoxCloneService, BoxService};
pub use boxed_local::{LocalBoxCloneService, LocalBoxService, RcService};
pub use ext::ServiceExt;
pub use map::Map;
pub use map_err::MapErr;
pub use map_ok::MapOk;
pub use map_request::MapRequest;
pub use or_else::OrElse;
pub use service_fn::{service_fn, ServiceFn};
pub use then::Then;

use std::{future::Future, rc::Rc, sync::Arc};

pub trait Service<Req> {
    type Response;
    type Error;
    type Future<'f>: Future<Output = Result<Self::Response, Self::Error>>
    where
        Self: 'f;

    fn call(&self, req: Req) -> Self::Future<'_>;
}

impl<'a, S, Req> Service<Req> for &'a mut S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f> where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (**self).call(req)
    }
}

impl<'a, S, Req> Service<Req> for &'a S
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f> where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (**self).call(req)
    }
}

impl<S, Req> Service<Req> for Box<S>
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f> where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (**self).call(req)
    }
}

impl<S, Req> Service<Req> for Rc<S>
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f> where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (**self).call(req)
    }
}

impl<S, Req> Service<Req> for Arc<S>
where
    S: Service<Req> + ?Sized,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f> where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (**self).call(req)
    }
}
