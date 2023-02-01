use std::fmt;

use echo_core::Request;
use serde::Deserialize;

pub fn query<'de, T>(req: &'de Request) -> Result<T, ExtractQueryError>
where
    T: Deserialize<'de>,
{
    let query = req.uri().query().unwrap_or_default();
    serde_urlencoded::from_str(query).map_err(ExtractQueryError::FailedToDeserialize)
}

#[derive(Debug)]
pub enum ExtractQueryError {
    FailedToDeserialize(serde_urlencoded::de::Error),
}

impl fmt::Display for ExtractQueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractQueryError::FailedToDeserialize(e) => {
                write!(f, "failed to deserialize query string ({e})")
            }
        }
    }
}

impl std::error::Error for ExtractQueryError {}
