use std::pin::Pin;
use std::task::{Context, Poll};

use echo_core::body::{Body as EchoBody, BodyExt, BoxBody, Bytes, Frame, SizeHint};
use echo_core::BoxError;
use hyper::body::{Body as HyperBody, Incoming};

pin_project_lite::pin_project! {
    pub(crate) struct HyperToEcho {
        #[pin]
        body: Incoming,
    }
}

impl HyperToEcho {
    pub(crate) fn to(body: Incoming) -> BoxBody {
        Self { body }.boxed()
    }
}

impl EchoBody for HyperToEcho {
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.project().body.poll_frame(cx).map_err(From::from)
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}

pin_project_lite::pin_project! {
    pub(crate) struct EchoToHyper {
        #[pin]
        body: BoxBody,
    }
}

impl EchoToHyper {
    pub(crate) fn to(body: BoxBody) -> Self {
        Self { body }
    }
}

impl HyperBody for EchoToHyper {
    type Data = Bytes;
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        self.project()
            .body
            .poll_frame(cx)
            .map_err(|e| e.to_string().into())
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}
