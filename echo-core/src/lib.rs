#![forbid(unsafe_code)]
#![deny(private_in_public)]
#![deny(unreachable_pub)]
#![warn(missing_debug_implementations)]

mod util;

pub mod body;
pub mod middleware;
pub mod response;
pub mod service;

pub mod http {
    pub use http::*;
}

pub type Request<B = body::BoxBody> = http::Request<B>;
pub type Response<B = body::BoxBody> = http::Response<B>;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
