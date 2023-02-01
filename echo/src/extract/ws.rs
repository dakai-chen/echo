use echo_core::Request;

use crate::ws::{WebSocketUpgrade, WebSocketUpgradeError};

pub fn ws(req: &mut Request) -> Result<WebSocketUpgrade, WebSocketUpgradeError> {
    WebSocketUpgrade::from_request(req)
}
