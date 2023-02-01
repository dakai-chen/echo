use std::{fmt, future::Future, pin::Pin, rc::Rc};

use super::Service;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub struct LocalBoxService<Req, Res, Err> {
    service: Box<dyn LocalAnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> LocalBoxService<Req, Res, Err> {
    #[inline]
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for LocalBoxService<Req, Res, Err>
where
    Req: 'static,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = LocalBoxFuture<'f, Result<Self::Response, Self::Error>>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        self.service.call(req)
    }
}

impl<Req, Res, Err> fmt::Debug for LocalBoxService<Req, Res, Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalBoxService").finish()
    }
}

pub struct LocalBoxCloneService<Req, Res, Err> {
    service: Box<dyn LocalCloneService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> LocalBoxCloneService<Req, Res, Err> {
    #[inline]
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + Clone + 'static,
    {
        Self {
            service: Box::new(service),
        }
    }
}

impl<Req, Res, Err> Clone for LocalBoxCloneService<Req, Res, Err> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone_box(),
        }
    }
}

impl<Req, Res, Err> Service<Req> for LocalBoxCloneService<Req, Res, Err>
where
    Req: 'static,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = LocalBoxFuture<'f, Result<Self::Response, Self::Error>>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        self.service.call(req)
    }
}

impl<Req, Res, Err> fmt::Debug for LocalBoxCloneService<Req, Res, Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalBoxCloneService").finish()
    }
}

pub struct RcService<Req, Res, Err> {
    service: Rc<dyn LocalAnyService<Req, Response = Res, Error = Err>>,
}

impl<Req, Res, Err> RcService<Req, Res, Err> {
    #[inline]
    pub fn new<S>(service: S) -> Self
    where
        S: Service<Req, Response = Res, Error = Err> + 'static,
    {
        Self {
            service: Rc::new(service),
        }
    }
}

impl<Req, Res, Err> Clone for RcService<Req, Res, Err> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            service: Rc::clone(&self.service),
        }
    }
}

impl<Req, Res, Err> Service<Req> for RcService<Req, Res, Err>
where
    Req: 'static,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = LocalBoxFuture<'f, Result<Self::Response, Self::Error>>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        self.service.call(req)
    }
}

impl<Req, Res, Err> fmt::Debug for RcService<Req, Res, Err> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RcService").finish()
    }
}

trait LocalAnyService<Req> {
    type Response;
    type Error;

    fn call<'a>(&'a self, req: Req) -> LocalBoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a;
}

impl<S, Req> LocalAnyService<Req> for S
where
    S: Service<Req>,
{
    type Response = S::Response;
    type Error = S::Error;

    fn call<'a>(&'a self, req: Req) -> LocalBoxFuture<'a, Result<Self::Response, Self::Error>>
    where
        Req: 'a,
    {
        Box::pin(Service::call(self, req))
    }
}

trait LocalCloneService<Req>: LocalAnyService<Req> {
    fn clone_box(
        &self,
    ) -> Box<dyn LocalCloneService<Req, Response = Self::Response, Error = Self::Error>>;
}

impl<S, Req> LocalCloneService<Req> for S
where
    S: Service<Req> + Clone + 'static,
{
    fn clone_box(
        &self,
    ) -> Box<dyn LocalCloneService<Req, Response = Self::Response, Error = Self::Error>> {
        Box::new(self.clone())
    }
}
