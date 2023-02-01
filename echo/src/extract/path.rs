use std::fmt;
use std::str::FromStr;

use echo_core::{BoxError, Request};

use crate::route::PathParams;

pub fn path<T>(req: &Request, name: &str) -> Result<T, ExtractPathError>
where
    T: FromStr,
    T::Err: Into<BoxError>,
{
    find(req, name).map_or_else(
        || Err(ExtractPathError::MissingParam { name: name.into() }),
        |param| {
            param
                .parse::<T>()
                .map_err(|e| ExtractPathError::InvalidParam {
                    name: name.into(),
                    source: e.into(),
                })
        },
    )
}

fn find<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    crate::extract::extension(req)
        .map(|params: &PathParams| params.get_ref())
        .and_then(|params| params.into_iter().rev().find(|(k, _)| k == name))
        .map(|(_, v)| v.as_str())
}

#[derive(Debug)]
pub enum ExtractPathError {
    MissingParam { name: String },
    InvalidParam { name: String, source: BoxError },
}

impl fmt::Display for ExtractPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractPathError::MissingParam { name } => {
                write!(f, "missing path param `{name}`")
            }
            ExtractPathError::InvalidParam { name, source } => {
                write!(f, "invalid path param `{name}` ({source})")
            }
        }
    }
}

impl std::error::Error for ExtractPathError {}
