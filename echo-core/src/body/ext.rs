use crate::BoxError;

use super::{Body, BodyStream, BoxBody, Collect, Limited, LocalBoxBody, MapErr, Next};

pub trait BodyExt: Body {
    fn next(&mut self) -> Next<'_, Self>
    where
        Self: Unpin,
    {
        Next(self)
    }

    fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
        F: FnMut(Self::Error) -> E,
    {
        MapErr::new(self, f)
    }

    fn stream(self) -> BodyStream<Self>
    where
        Self: Sized,
    {
        BodyStream::new(self)
    }

    fn limit(self, limit: usize) -> Limited<Self>
    where
        Self: Sized,
    {
        Limited::new(self, limit)
    }

    fn collect(self) -> Collect<Self>
    where
        Self: Sized,
    {
        Collect::new(self)
    }

    fn boxed(self) -> BoxBody
    where
        Self: Sized + Send + 'static,
        Self::Error: Into<BoxError>,
    {
        BoxBody::new(self)
    }

    fn boxed_local(self) -> LocalBoxBody
    where
        Self: Sized + 'static,
        Self::Error: Into<BoxError>,
    {
        LocalBoxBody::new(self)
    }
}

impl<B> BodyExt for B where B: Body {}
