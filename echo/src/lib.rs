#![forbid(unsafe_code)]
#![deny(private_in_public)]
#![deny(unreachable_pub)]
#![warn(missing_debug_implementations)]

pub use echo_core::*;

mod util;

pub mod extract;
pub mod middleware;
pub mod response;
pub mod route;

#[cfg(feature = "macros")]
pub use echo_macros::route;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "ws")]
pub mod ws;
