use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::BoxError;

use super::{Body, BodyExt, Bytes, Frame, SizeHint};

pub struct LocalBoxBody {
    body: Pin<Box<dyn Body<Error = BoxError>>>,
}

impl LocalBoxBody {
    #[inline]
    pub fn new<B>(body: B) -> Self
    where
        B: Body + 'static,
        B::Error: Into<BoxError>,
    {
        Self {
            body: Box::pin(body.map_err(Into::into)),
        }
    }
}

impl Default for LocalBoxBody {
    fn default() -> Self {
        LocalBoxBody::new(())
    }
}

impl Body for LocalBoxBody {
    type Error = BoxError;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        self.body.as_mut().poll_frame(cx)
    }

    fn size_hint(&self) -> SizeHint {
        self.body.size_hint()
    }
}

impl fmt::Debug for LocalBoxBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalBoxBody").finish()
    }
}
