use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use super::Service;

#[derive(Clone, Copy)]
pub struct AndThen<S, F> {
    service: S,
    f: F,
}

impl<S, F> AndThen<S, F> {
    #[inline]
    pub fn new(service: S, f: F) -> Self {
        Self { service, f }
    }
}

impl<S, F, Fut, Req, Res> Service<Req> for AndThen<S, F>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Fut,
    Fut: Future<Output = Result<Res, S::Error>>,
{
    type Response = Res;
    type Error = S::Error;
    type Future<'f> = AndThenFuture<'f, S, F, Fut, Req>
    where
        Self: 'f;

    #[inline]
    fn call(&self, req: Req) -> Self::Future<'_> {
        AndThenFuture {
            slf: self,
            state: State::A {
                fut: self.service.call(req),
            },
        }
    }
}

impl<S, F> fmt::Debug for AndThen<S, F>
where
    S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AndThen")
            .field("service", &self.service)
            .field("f", &std::any::type_name::<F>())
            .finish()
    }
}

pin_project_lite::pin_project! {
    pub struct AndThenFuture<'f, S, F, Fut, Req>
    where
        S: Service<Req>,
    {
        slf: &'f AndThen<S, F>,
        #[pin]
        state: State<S::Future<'f>, Fut>,
    }
}

impl<'f, S, F, Fut, Req, Res> Future for AndThenFuture<'f, S, F, Fut, Req>
where
    S: Service<Req>,
    F: Fn(S::Response) -> Fut,
    Fut: Future<Output = Result<Res, S::Error>>,
{
    type Output = Result<Res, S::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.as_mut().project();
        match this.state.as_mut().project() {
            StateProj::A { fut } => match fut.poll(cx) {
                Poll::Ready(Ok(res)) => {
                    this.state.set(State::B {
                        fut: (this.slf.f)(res),
                    });
                    self.poll(cx)
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
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
