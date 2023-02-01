use echo_core::body::{Body, BodyExt};
use echo_core::http::{header, HeaderValue};
use echo_core::response::IntoResponse;
use echo_core::{BoxError, Response};

#[derive(Debug, Clone, Copy)]
pub struct Html<B>(pub B);

impl<B> IntoResponse for Html<B>
where
    B: Body + Send + 'static,
    B::Error: Into<BoxError>,
{
    fn into_response(self) -> Response {
        let mut res = Response::new(self.0.boxed());
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        );
        res
    }
}
