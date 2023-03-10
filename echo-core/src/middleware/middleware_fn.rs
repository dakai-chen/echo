use std::fmt;

use super::Middleware;

#[inline]
pub fn middleware_fn<F>(f: F) -> MiddlewareFn<F> {
    MiddlewareFn { f }
}

#[derive(Clone, Copy)]
pub struct MiddlewareFn<F> {
    f: F,
}

impl<F, S1, S2> Middleware<S1> for MiddlewareFn<F>
where
    F: FnOnce(S1) -> S2,
{
    type Service = S2;

    #[inline]
    fn transform(self, service: S1) -> Self::Service {
        (self.f)(service)
    }
}

impl<F> fmt::Debug for MiddlewareFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiddlewareFn")
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}
