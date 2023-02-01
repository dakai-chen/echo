use std::fmt;
use std::str::FromStr;

use echo_core::http::header::HeaderName;
use echo_core::{BoxError, Request};

pub fn header<T, N>(req: &Request, name: N) -> Result<T, ExtractHeaderError>
where
    T: FromStr,
    T::Err: Into<BoxError>,
    N: TryInto<HeaderName>,
    N::Error: Into<BoxError>,
{
    let name = name
        .try_into()
        .map_err(|e| ExtractHeaderError::InvalidHeaderName { source: e.into() })?;

    if let Some(value) = req.headers().get(&name) {
        match value.to_str() {
            Ok(s) => s
                .parse::<T>()
                .map_err(|e| ExtractHeaderError::InvalidHeader {
                    name,
                    source: e.into(),
                }),
            Err(e) => Err(ExtractHeaderError::InvalidHeader {
                name,
                source: e.into(),
            }),
        }
    } else {
        Err(ExtractHeaderError::MissingHeader { name })
    }
}

#[derive(Debug)]
pub enum ExtractHeaderError {
    MissingHeader { name: HeaderName },
    InvalidHeader { name: HeaderName, source: BoxError },
    InvalidHeaderName { source: BoxError },
}

impl fmt::Display for ExtractHeaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractHeaderError::MissingHeader { name } => {
                write!(f, "missing request header `{name}`")
            }
            ExtractHeaderError::InvalidHeader { name, source } => {
                write!(f, "invalid request header `{name}` ({source})")
            }
            ExtractHeaderError::InvalidHeaderName { source } => {
                write!(f, "invalid request header name ({source})")
            }
        }
    }
}

impl std::error::Error for ExtractHeaderError {}
