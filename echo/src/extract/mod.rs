mod bytes;
mod extension;
mod form;
mod header;
mod json;
mod path;
mod query;
mod stream;

pub use self::bytes::bytes;
pub use extension::{extension, extension_mut};
pub use form::{form, ExtractFormError};
pub use header::{header, ExtractHeaderError};
pub use json::{json, ExtractJsonError};
pub use path::{path, ExtractPathError};
pub use query::{query, ExtractQueryError};
pub use stream::stream;

#[cfg(feature = "multipart")]
pub mod multipart;
#[cfg(feature = "multipart")]
pub use multipart::multipart;

#[cfg(feature = "ws")]
mod ws;
#[cfg(feature = "ws")]
pub use ws::ws;
