use std::fmt;

use echo_core::middleware::Middleware;
use echo_core::service::Service;
use echo_core::Request;

#[inline]
pub fn add_extension<F>(f: F) -> AddExtensionMiddleware<F> {
    AddExtensionMiddleware::new(f)
}

#[derive(Clone, Copy)]
pub struct AddExtensionMiddleware<F> {
    f: F,
}

impl<F> AddExtensionMiddleware<F> {
    #[inline]
    pub fn new(f: F) -> Self {
        Self { f }
    }
}

impl<S, F> Middleware<S> for AddExtensionMiddleware<F> {
    type Service = AddExtension<S, F>;

    fn transform(self, service: S) -> Self::Service {
        AddExtension { service, f: self.f }
    }
}

impl<F> fmt::Debug for AddExtensionMiddleware<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddExtensionMiddleware")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct AddExtension<S, F> {
    service: S,
    f: F,
}

impl<S, F, B, T> Service<Request<B>> for AddExtension<S, F>
where
    S: Service<Request<B>>,
    F: Fn() -> T,
    T: Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f>
    where
        Self: 'f;

    fn call(&self, mut req: Request<B>) -> Self::Future<'_> {
        req.extensions_mut().insert((self.f)());
        self.service.call(req)
    }
}

impl<S, F> fmt::Debug for AddExtension<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddExtension")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
