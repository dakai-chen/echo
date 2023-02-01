use echo_core::body::BodyExt;
use echo_core::http::{header, HeaderValue, StatusCode};
use echo_core::response::IntoResponse;
use echo_core::Response;
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match serde_json::to_vec(&self.0) {
            Ok(bytes) => {
                let mut res = Response::new(bytes.boxed());
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
                );
                res
            }
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
