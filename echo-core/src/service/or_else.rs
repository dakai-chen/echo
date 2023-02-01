use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct OrElse<S, F> {
    service: S,
    f: F,
}

impl<S, F> OrElse<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Err> Service<Req> for OrElse<S, F>
where
    S: Service<Req>,
    F: Fn(S::Error) -> Fut,
    Fut: Future<Output = Result<S::Response, Err>>,
{
    type Response = S::Response;
    type Error = Err;
    type Future<'f> = OrElseFuture<'f, S, F, Fut, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        OrElseFuture {
            slf: self,
            state: State::A {
                fut: self.service.call(req),
            },
        }
    }
}

impl<S, F> fmt::Debug for OrElse<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrElse")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct OrElseFuture<'f, S, F, Fut, Req>
    where
        S: Service<Req>,
    {
        slf: &'f OrElse<S, F>,
        #[pin]
        state: State<S::Future<'f>, Fut>,
    }
}

impl<'f, S, F, Fut, Req, Err> Future for OrElseFuture<'f, S, F, Fut, Req>
where
    S: Service<Req>,
    F: Fn(S::Error) -> Fut,
    Fut: Future<Output = Result<S::Response, Err>>,
{
    type Output = Result<S::Response, Err>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        match this.state.as_mut().project() {
            StateProj::A { fut } => match fut.poll(cx) {
                Poll::Ready(Ok(res)) => Poll::Ready(Ok(res)),
                Poll::Ready(Err(err)) => {
                    this.state.set(State::B {
                        fut: (this.slf.f)(err),
                    });
                    self.poll(cx)
                }
                Poll::Pending => Poll::Pending,
            },
            StateProj::B { fut } => fut.poll(cx).map(|r| {
                this.state.set(State::Empty);
                r
            }),
            StateProj::Empty => panic!("future must not be polled after it returned `Poll::Ready`"),
        }
    }
}

pin_project_lite::pin_project! {
    #[project = StateProj]
    enum State<Fut1, Fut2> {
        A { #[pin] fut: Fut1 },
        B { #[pin] fut: Fut2 },
        Empty,
    }
}
