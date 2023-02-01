use std::fmt;
use std::pin::Pin;
use std::task::{Context, Poll};

use echo_core::body::{Body, BodyExt, Bytes};
use echo_core::http::header::{HeaderMap, CONTENT_TYPE};
use echo_core::{BoxError, Request};
use futures_util::{Stream, TryStreamExt};

pub fn multipart(req: &mut Request) -> Result<Multipart, MultipartError> {
    Multipart::from_request(req)
}

pub struct Multipart {
    inner: multer::Multipart<'static>,
}

impl Multipart {
    pub fn from_request<B>(req: &mut Request<B>) -> Result<Self, MultipartError>
    where
        B: Body + Default + Send + 'static,
        B::Error: Into<BoxError>,
    {
        let boundary =
            Self::parse_boundary(req.headers()).ok_or(MultipartError::UnsupportedContentType)?;

        let stream = std::mem::take(req.body_mut())
            .stream()
            .try_filter_map(|frame| async move { Ok(frame.into_data().ok()) });

        Ok(Self {
            inner: multer::Multipart::new(stream, boundary),
        })
    }

    fn parse_boundary(headers: &HeaderMap) -> Option<String> {
        headers
            .get(&CONTENT_TYPE)?
            .to_str()
            .ok()
            .and_then(|content_type| multer::parse_boundary(content_type).ok())
    }
}

impl Multipart {
    pub async fn next(&mut self) -> Result<Option<Field<'_>>, MultipartError> {
        let field = self
            .inner
            .next_field()
            .await
            .map_err(MultipartError::from_multer)?;

        Ok(field.map(|f| Field {
            inner: f,
            _multipart: self,
        }))
    }
}

impl fmt::Debug for Multipart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Multipart").finish()
    }
}

pub struct Field<'a> {
    inner: multer::Field<'static>,
    _multipart: &'a mut Multipart,
}

impl<'a> Field<'a> {
    pub fn name(&self) -> Option<&str> {
        self.inner.name()
    }

    pub fn file_name(&self) -> Option<&str> {
        self.inner.file_name()
    }

    pub fn content_type(&self) -> Option<&str> {
        self.inner.content_type().map(|m| m.as_ref())
    }

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub async fn bytes(self) -> Result<Bytes, MultipartError> {
        self.inner
            .bytes()
            .await
            .map_err(MultipartError::from_multer)
    }

    pub async fn text(self) -> Result<String, MultipartError> {
        self.inner.text().await.map_err(MultipartError::from_multer)
    }

    pub async fn chunk(&mut self) -> Result<Option<Bytes>, MultipartError> {
        self.inner
            .chunk()
            .await
            .map_err(MultipartError::from_multer)
    }
}

impl<'a> Stream for Field<'a> {
    type Item = Result<Bytes, MultipartError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner)
            .poll_next(cx)
            .map_err(MultipartError::from_multer)
    }
}

impl<'a> fmt::Debug for Field<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field").finish()
    }
}

#[derive(Debug)]
pub enum MultipartError {
    UnsupportedContentType,
    Other(BoxError),
}

impl MultipartError {
    fn from_multer(error: multer::Error) -> Self {
        MultipartError::Other(error.into())
    }
}

impl fmt::Display for MultipartError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MultipartError::UnsupportedContentType => f.write_str("unsupported content type"),
            MultipartError::Other(e) => {
                write!(f, "error parsing `multipart/form-data` request ({e})")
            }
        }
    }
}

impl std::error::Error for MultipartError {}
