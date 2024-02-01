pub mod message;
pub mod upgrade;
pub mod websocket;

use std::fmt::Display;

use axum_core::body::Body;
use axum_core::response::Response;
use http::StatusCode;

pub use crate::{message::Message, upgrade::WebSocketUpgrade, websocket::WebSocket};

#[derive(Debug)]
pub enum WebSocketError {
    TokioWebSocket(tokio_websockets::Error),
    Hyper(hyper::Error),
    MethodNotGet,
    InvalidConnectionHeader,
    InvalidUpgradeHeader,
    InvalidWebSocketVersionHeader,
    UpgradeFailure,
}

impl Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WebSocket error: {}", self)
    }
}

impl std::error::Error for WebSocketError {}

use axum_core::response::IntoResponse;
impl IntoResponse for WebSocketError {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }
}

impl From<hyper::Error> for WebSocketError {
    fn from(e: hyper::Error) -> Self {
        WebSocketError::Hyper(e)
    }
}

impl From<tokio_websockets::Error> for WebSocketError {
    fn from(e: tokio_websockets::Error) -> Self {
        WebSocketError::TokioWebSocket(e)
    }
}
