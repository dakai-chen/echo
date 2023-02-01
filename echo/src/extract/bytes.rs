use echo_core::body::{BodyExt, Bytes};
use echo_core::{BoxError, Request};

pub async fn bytes(req: &mut Request) -> Result<Bytes, BoxError> {
    Ok(std::mem::take(req.body_mut()).collect().await?.to_bytes())
}
