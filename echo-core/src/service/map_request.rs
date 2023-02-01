use std::fmt;

use super::Service;

#[derive(Clone, Copy)]
pub struct MapRequest<S, F> {
    service: S,
    f: F,
}

impl<S, F> MapRequest<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, R1, R2> Service<R1> for MapRequest<S, F>
where
    S: Service<R2>,
    F: Fn(R1) -> R2,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future<'f> = S::Future<'f>
    where
        Self: 'f;

    fn call(&self, req: R1) -> Self::Future<'_> {
        self.service.call((self.f)(req))
    }
}

impl<S, F> fmt::Debug for MapRequest<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MapRequest")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
