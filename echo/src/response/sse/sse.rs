use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use echo_core::body::{Body, BodyExt, Bytes, Frame};
use echo_core::http::header;
use echo_core::response::IntoResponse;
use echo_core::{BoxError, Response};
use futures_util::Stream;

use super::{
    keep_alive::{KeepAlive, KeepAliveStream},
    Event,
};

#[derive(Clone)]
pub struct Sse<S> {
    stream: S,
    keep_alive: Option<KeepAlive>,
}

impl<S> Sse<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            keep_alive: None,
        }
    }

    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

impl<S> fmt::Debug for Sse<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sse")
            .field("stream", &format_args!("{}", std::any::type_name::<S>()))
            .field("keep_alive", &self.keep_alive)
            .finish()
    }
}

impl<S, E> IntoResponse for Sse<S>
where
    S: Stream<Item = Result<Event, E>> + Send + 'static,
    E: Into<BoxError>,
{
    fn into_response(self) -> Response {
        let body = SseBody {
            stream: self.stream,
            keep_alive: self.keep_alive.map(KeepAliveStream::new),
        };

        Response::builder()
            .header(header::CONTENT_TYPE, mime::TEXT_EVENT_STREAM.as_ref())
            .header(header::CACHE_CONTROL, "no-cache")
            .body(body.boxed())
            .unwrap()
    }
}

pin_project_lite::pin_project! {
    struct SseBody<S> {
        #[pin]
        stream: S,
        #[pin]
        keep_alive: Option<KeepAliveStream>,
    }
}

impl<S, E> Body for SseBody<S>
where
    S: Stream<Item = Result<Event, E>>,
{
    type Error = E;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let this = self.project();

        match this.stream.poll_next(cx) {
            Poll::Pending => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive
                        .poll_event(cx)
                        .map(|e| Some(Ok(Frame::data(Bytes::from(e.to_string())))))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Ok(event))) => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.reset();
                }
                Poll::Ready(Some(Ok(Frame::data(Bytes::from(event.to_string())))))
            }
            Poll::Ready(Some(Err(error))) => Poll::Ready(Some(Err(error))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}
