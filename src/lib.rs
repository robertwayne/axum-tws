#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

pub mod upgrade;
pub mod websocket;

use std::fmt::Display;

use axum_core::body::Body;
use axum_core::response::IntoResponse;
use axum_core::response::Response;
use http::StatusCode;

pub use tokio_websockets::*;

pub use crate::{upgrade::WebSocketUpgrade, websocket::WebSocket};

#[derive(Debug)]
pub enum WebSocketError {
    ConnectionNotUpgradeable,
    Internal(tokio_websockets::Error),
    InvalidConnectionHeader,
    /// For WebSocket over HTTP/2+
    InvalidProtocolPseudoheader,
    InvalidUpgradeHeader,
    InvalidWebSocketVersionHeader,
    /// Invalid method for WebSocket over HTTP/1.x
    MethodNotGet,
    /// Invalid method for WebSocket over HTTP/2+
    MethodNotConnect,
    UpgradeFailed(hyper::Error),
}

impl Display for WebSocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebSocketError::ConnectionNotUpgradeable => {
                write!(f, "connection is not upgradeable")
            }
            WebSocketError::Internal(e) => {
                write!(f, "internal server error: {}", e)
            }
            WebSocketError::InvalidConnectionHeader => {
                write!(f, "invalid `Connection` header")
            }
            WebSocketError::InvalidProtocolPseudoheader => {
                write!(f, "invalid `:protocol` pseudoheader")
            }
            WebSocketError::InvalidUpgradeHeader => {
                write!(f, "invalid `Upgrade` header")
            }
            WebSocketError::InvalidWebSocketVersionHeader => {
                write!(f, "invalid `Sec-WebSocket-Version` header")
            }
            WebSocketError::MethodNotGet => {
                write!(f, "http request method must be `GET`")
            }
            WebSocketError::MethodNotConnect => {
                write!(f, "http2 request method must be `CONNECT`")
            }
            WebSocketError::UpgradeFailed(e) => {
                write!(f, "upgrade failed: {}", e)
            }
        }
    }
}

impl std::error::Error for WebSocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WebSocketError::Internal(e) => Some(e),
            WebSocketError::UpgradeFailed(e) => Some(e),
            _ => None,
        }
    }
}

impl IntoResponse for WebSocketError {
    fn into_response(self) -> Response<Body> {
        let status = match self {
            WebSocketError::ConnectionNotUpgradeable => StatusCode::UPGRADE_REQUIRED,

            // Request headers are invalid or missing.
            WebSocketError::InvalidConnectionHeader
            | WebSocketError::InvalidUpgradeHeader
            | WebSocketError::InvalidWebSocketVersionHeader => StatusCode::BAD_REQUEST,

            // Invalid request method.
            WebSocketError::MethodNotGet => StatusCode::METHOD_NOT_ALLOWED,

            // All other errors will be treated as internal server errors.
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap()
    }
}

impl From<tokio_websockets::Error> for WebSocketError {
    fn from(e: tokio_websockets::Error) -> Self {
        WebSocketError::Internal(e)
    }
}

impl From<hyper::Error> for WebSocketError {
    fn from(e: hyper::Error) -> Self {
        WebSocketError::UpgradeFailed(e)
    }
}
