use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use echo_core::http::uri::{Parts, Uri};
use echo_core::middleware::{middleware_fn, Middleware};
use echo_core::response::IntoResponse;
use echo_core::service::future::BoxFuture;
use echo_core::service::{ArcService, Service, ServiceExt};
use echo_core::{BoxError, Request, Response};
use matchit::{Match, MatchError};

use super::future::RouteFuture;
use super::method::{MergeToMethodRouter, MethodRouter};
use super::{IntoMethodRoute, MethodRoute, RouteError, RouterError};

pub(crate) const PRIVATE_TAIL_PARAM: &'static str = "__private__tail_param";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct RouteId(u32);

impl RouteId {
    fn next(mut self) -> Option<Self> {
        self.0.checked_add(1).map(|id| {
            self.0 = id;
            self
        })
    }
}

#[derive(Default)]
struct RouterInner {
    id: RouteId,
    inner: matchit::Router<RouteId>,
    id_to_path: HashMap<RouteId, Arc<str>>,
    path_to_id: HashMap<Arc<str>, RouteId>,
}

impl RouterInner {
    fn at<'m, 'p>(&'m self, path: &'p str) -> Result<Match<'m, 'p, &'m RouteId>, MatchError> {
        self.inner.at(path)
    }

    fn find(&self, path: &str) -> Option<RouteId> {
        self.path_to_id.get(path).copied()
    }

    fn next(&mut self) -> Option<RouteId> {
        self.id.next().map(|id| {
            self.id = id;
            id
        })
    }

    fn add(&mut self, path: String) -> Result<RouteId, RouterError> {
        let id = self.next().ok_or_else(|| RouterError::TooManyPath)?;

        if let Err(e) = self.inner.insert(&path, id) {
            return Err(RouterError::from_insert_error(path, e));
        }

        let path: Arc<str> = path.into();
        self.id_to_path.insert(id, path.clone());
        self.path_to_id.insert(path, id);

        Ok(id)
    }
}

enum Endpoint<T> {
    Route(T),
    Scope(T),
}

#[derive(Default)]
pub struct Router {
    inner: RouterInner,
    table: HashMap<RouteId, Endpoint<MethodRouter>>,
}

impl Router {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn route<S>(self, path: &str, service: S) -> Self
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + Send + Sync + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
        for<'f> <S::Service as Service<Request>>::Future<'f>: Send,
    {
        self.try_route(path, service).unwrap()
    }

    pub fn try_route<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + Send + Sync + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
        for<'f> <S::Service as Service<Request>>::Future<'f>: Send,
    {
        if !path.starts_with('/') {
            return Err(RouterError::InvalidPath {
                path: path.to_owned(),
                message: format!("path must start with a `/`"),
            });
        }
        let path = if path.ends_with('*') {
            format!("{path}{PRIVATE_TAIL_PARAM}")
        } else {
            path.into()
        };
        self.add_route(
            path,
            Endpoint::Route(
                service
                    .into_method_route()
                    .with(middleware_fn(Self::into_arc_service)),
            ),
        )
    }

    pub fn scope<S>(self, path: &str, service: S) -> Self
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + Send + Sync + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
        for<'f> <S::Service as Service<Request>>::Future<'f>: Send,
    {
        self.try_scope(path, service).unwrap()
    }

    pub fn try_scope<S>(self, path: &str, service: S) -> Result<Self, RouterError>
    where
        S: IntoMethodRoute,
        S::Service: Service<Request> + Send + Sync + 'static,
        <S::Service as Service<Request>>::Response: IntoResponse,
        <S::Service as Service<Request>>::Error: Into<BoxError>,
        for<'f> <S::Service as Service<Request>>::Future<'f>: Send,
    {
        if !path.starts_with('/') {
            return Err(RouterError::InvalidPath {
                path: path.to_owned(),
                message: format!("path must start with a `/`"),
            });
        }
        let path = if path.ends_with('/') {
            format!("{path}*{PRIVATE_TAIL_PARAM}")
        } else {
            format!("{path}/*{PRIVATE_TAIL_PARAM}")
        };
        self.add_route(
            path,
            Endpoint::Scope(
                service
                    .into_method_route()
                    .with(middleware_fn(Self::into_arc_service)),
            ),
        )
    }

    pub fn mount<S>(self, route: impl Into<Route<S>>) -> Self
    where
        S: Service<Request> + Send + Sync + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
        for<'f> S::Future<'f>: Send,
    {
        self.try_mount(route).unwrap()
    }

    pub fn try_mount<S>(self, route: impl Into<Route<S>>) -> Result<Self, RouterError>
    where
        S: Service<Request> + Send + Sync + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
        for<'f> S::Future<'f>: Send,
    {
        route.into().mount_to(self)
    }

    pub fn merge(self, other: Router) -> Self {
        self.try_merge(other).unwrap()
    }

    pub fn try_merge(mut self, other: Router) -> Result<Self, RouterError> {
        for (id, endpoint) in other.table {
            self = self.add_route(other.inner.id_to_path[&id].as_ref().to_owned(), endpoint)?;
        }
        Ok(self)
    }

    fn add_route<T: MergeToMethodRouter>(
        mut self,
        path: String,
        endpoint: Endpoint<T>,
    ) -> Result<Self, RouterError> {
        let id = self.add_path(path.clone())?;

        let result = match endpoint {
            Endpoint::Route(service) => {
                let Endpoint::Route(router) = self.table.entry(id).or_insert_with(|| Endpoint::Route(Default::default())) else {
                    return Err(RouterError::Conflict {
                        path,
                        message: format!("conflict with previously registered route"),
                    })
                };
                service.merge_to(router)
            }
            Endpoint::Scope(service) => {
                let Endpoint::Scope(router) = self.table.entry(id).or_insert_with(|| Endpoint::Scope(Default::default())) else {
                    return Err(RouterError::Conflict {
                        path,
                        message: format!("conflict with previously registered route"),
                    })
                };
                service.merge_to(router)
            }
        };

        result.map_err(|method| {
            let message = match method {
                Some(method) => {
                    format!("conflict with previously registered `{method}` HTTP method")
                }
                None => format!("conflict with previously registered any HTTP method"),
            };
            RouterError::Conflict { path, message }
        })?;

        Ok(self)
    }

    fn add_path(&mut self, path: String) -> Result<RouteId, RouterError> {
        let id = if let Some(id) = self.inner.find(&path) {
            id
        } else {
            self.inner.add(path)?
        };
        Ok(id)
    }

    fn into_arc_service<S>(service: S) -> ArcService<Request, Response, BoxError>
    where
        S: Service<Request> + Send + Sync + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
        for<'f> S::Future<'f>: Send,
    {
        crate::util::try_downcast(service).unwrap_or_else(|service| {
            service
                .map_ok(IntoResponse::into_response)
                .map_err(Into::into)
                .boxed_arc()
        })
    }
}

impl fmt::Debug for Router {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Router").finish()
    }
}

impl Service<Request> for Router {
    type Response = Response;
    type Error = BoxError;
    type Future<'f> = RouteFuture<BoxFuture<'f, Result<Response, BoxError>>>
    where
        Self: 'f;

    fn call(&self, mut req: Request) -> Self::Future<'_> {
        match self.inner.at(req.uri().path()) {
            Ok(Match { value, params }) => {
                let (params, tail) = super::params::prase_path_params(params);
                super::params::insert_path_params(req.extensions_mut(), params);
                match self.table.get(value) {
                    Some(Endpoint::Route(service)) => service.call(req),
                    Some(Endpoint::Scope(service)) => {
                        replace_request_path(&mut req, &tail.unwrap());
                        service.call(req)
                    }
                    None => RouteFuture::B {
                        err: Some(RouteError::not_found(req).into()),
                    },
                }
            }
            Err(_) => RouteFuture::B {
                err: Some(RouteError::not_found(req).into()),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Route<S> {
    path: String,
    service: MethodRoute<S>,
}

impl<S> Route<S> {
    pub fn new<T>(path: impl Into<String>, service: T) -> Self
    where
        T: IntoMethodRoute<Service = S>,
    {
        Self {
            path: path.into(),
            service: service.into_method_route(),
        }
    }

    pub fn with<T>(self, middleware: T) -> Route<T::Service>
    where
        T: Middleware<S>,
    {
        Route {
            path: self.path,
            service: self.service.with(middleware),
        }
    }

    fn mount_to(self, router: Router) -> Result<Router, RouterError>
    where
        S: Service<Request> + Send + Sync + 'static,
        S::Response: IntoResponse,
        S::Error: Into<BoxError>,
        for<'f> S::Future<'f>: Send,
    {
        router.try_route(&self.path, self.service)
    }
}

fn replace_request_path(req: &mut Request, path: &str) {
    let uri = req.uri_mut();

    let path = if path.starts_with('/') {
        path[1..].as_ref()
    } else {
        path
    };

    let path_and_query = if let Some(query) = uri.query() {
        format!("/{path}?{query}")
    } else {
        format!("/{path}")
    };

    let mut parts = Parts::default();

    parts.scheme = uri.scheme().cloned();
    parts.authority = uri.authority().cloned();
    parts.path_and_query = Some(path_and_query.parse().unwrap());

    *uri = Uri::from_parts(parts).unwrap();
}
