use std::fmt;

use echo_core::http::{header, Method};
use echo_core::{BoxError, Request};
use serde::de::DeserializeOwned;

use super::query::ExtractQueryError;

pub async fn form<T>(req: &mut Request) -> Result<T, ExtractFormError>
where
    T: DeserializeOwned,
{
    if req.method() == Method::GET {
        crate::extract::query(req).map_err(ExtractFormError::from_extract_query_error)
    } else {
        if !has_content_type(req, &mime::APPLICATION_WWW_FORM_URLENCODED) {
            return Err(ExtractFormError::UnsupportedContentType);
        }

        let bytes = crate::extract::bytes(req)
            .await
            .map_err(|e| ExtractFormError::FailedToReadBody(e.into()))?;

        serde_urlencoded::from_bytes(&bytes).map_err(ExtractFormError::FailedToDeserialize)
    }
}

fn has_content_type(req: &Request, expected_content_type: &mime::Mime) -> bool {
    let content_type = if let Some(content_type) = req.headers().get(header::CONTENT_TYPE) {
        content_type
    } else {
        return false;
    };

    let content_type = if let Ok(content_type) = content_type.to_str() {
        content_type
    } else {
        return false;
    };

    content_type.starts_with(expected_content_type.as_ref())
}

#[derive(Debug)]
pub enum ExtractFormError {
    UnsupportedContentType,
    FailedToReadBody(BoxError),
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl ExtractFormError {
    fn from_extract_query_error(error: ExtractQueryError) -> Self {
        match error {
            ExtractQueryError::FailedToDeserialize(e) => ExtractFormError::FailedToDeserialize(e),
        }
    }
}

impl fmt::Display for ExtractFormError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractFormError::UnsupportedContentType => f.write_str("unsupported content type"),
            ExtractFormError::FailedToReadBody(e) => write!(f, "failed to read body ({e})"),
            ExtractFormError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize ({e})")
            }
        }
    }
}

impl std::error::Error for ExtractFormError {}
