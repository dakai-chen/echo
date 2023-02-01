use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct Then<S, F> {
    service: S,
    f: F,
}

impl<S, F> Then<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res, Err> Service<Req> for Then<S, F>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    type Response = Res;
    type Error = Err;
    type Future<'f> = ThenFuture<'f, S, F, Fut, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        ThenFuture {
            slf: self,
            state: State::A {
                fut: self.service.call(req),
            },
        }
    }
}

impl<S, F> fmt::Debug for Then<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Then")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct ThenFuture<'f, S, F, Fut, Req>
    where
        S: Service<Req>,
    {
        slf: &'f Then<S, F>,
        #[pin]
        state: State<S::Future<'f>, Fut>,
    }
}

impl<'f, S, F, Fut, Req, Res, Err> Future for ThenFuture<'f, S, F, Fut, Req>
where
    S: Service<Req>,
    F: Fn(Result<S::Response, S::Error>) -> Fut,
    Fut: Future<Output = Result<Res, Err>>,
{
    type Output = Result<Res, Err>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        match this.state.as_mut().project() {
            StateProj::A { fut } => match fut.poll(cx) {
                Poll::Ready(res) => {
                    this.state.set(State::B {
                        fut: (this.slf.f)(res),
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
