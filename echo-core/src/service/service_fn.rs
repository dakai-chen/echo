use std::{fmt, future::Future};

use super::Service;

#[inline]
pub fn service_fn<F>(f: F) -> ServiceFn<F> {
    ServiceFn { f }
}

#[derive(Clone, Copy)]
pub struct ServiceFn<F> {
    f: F,
}

impl<F, Fut, Req, Res, Err> Service<Req> for ServiceFn<F>
where
    F: Fn(Req) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = Fut where Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        (self.f)(req)
    }
}

impl<F> fmt::Debug for ServiceFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServiceFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
