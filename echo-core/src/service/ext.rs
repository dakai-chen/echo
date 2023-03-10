use crate::middleware::Middleware;

use super::{
    AndThen, ArcService, BoxCloneService, BoxService, LocalBoxCloneService, LocalBoxService, Map,
    MapErr, MapOk, MapRequest, OrElse, RcService, Service, Then,
};

pub trait ServiceExt<Req>: Service<Req> {
    fn with<T>(self, middleware: T) -> T::Service
    where
        Self: Sized,
        T: Middleware<Self>,
    {
        middleware.transform(self)
    }

    fn then<F>(self, f: F) -> Then<Self, F>
    where
        Self: Sized,
    {
        Then::new(self, f)
    }

    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen::new(self, f)
    }

    fn or_else<F>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
    {
        OrElse::new(self, f)
    }

    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }

    fn map_ok<F>(self, f: F) -> MapOk<Self, F>
    where
        Self: Sized,
    {
        MapOk::new(self, f)
    }

    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr::new(self, f)
    }

    fn map_request<F>(self, f: F) -> MapRequest<Self, F>
    where
        Self: Sized,
    {
        MapRequest::new(self, f)
    }

    fn boxed(self) -> BoxService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Send + Sync + 'static,
        for<'f> Self::Future<'f>: Send,
    {
        BoxService::new(self)
    }

    fn boxed_local(self) -> LocalBoxService<Req, Self::Response, Self::Error>
    where
        Self: Sized + 'static,
    {
        LocalBoxService::new(self)
    }

    fn boxed_clone(self) -> BoxCloneService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Clone + Send + Sync + 'static,
        for<'f> Self::Future<'f>: Send,
    {
        BoxCloneService::new(self)
    }

    fn boxed_clone_local(self) -> LocalBoxCloneService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Clone + 'static,
    {
        LocalBoxCloneService::new(self)
    }

    fn boxed_arc(self) -> ArcService<Req, Self::Response, Self::Error>
    where
        Self: Sized + Send + Sync + 'static,
        for<'f> Self::Future<'f>: Send,
    {
        ArcService::new(self)
    }

    fn boxed_rc(self) -> RcService<Req, Self::Response, Self::Error>
    where
        Self: Sized + 'static,
    {
        RcService::new(self)
    }
}

impl<S, Req> ServiceExt<Req> for S where S: Service<Req> {}
