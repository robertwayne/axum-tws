use std::future::Future;

use axum_core::body::Body;
use axum_core::extract::FromRequestParts;
use axum_core::response::Response;
use http::request::Parts;
use http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Version};
use hyper_util::rt::TokioIo;
use sha1::Digest;
use tokio_websockets::{Config, Limits};

use crate::{websocket::WebSocket, WebSocketError};

pub trait OnFailedUpgrade: Send + 'static {
    fn call(self, error: WebSocketError);
}

impl<F> OnFailedUpgrade for F
where
    F: FnOnce(WebSocketError) + Send + 'static,
{
    fn call(self, error: WebSocketError) {
        self(error)
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub struct DefaultOnFailedUpgrade;

impl OnFailedUpgrade for DefaultOnFailedUpgrade {
    #[inline]
    fn call(self, _error: WebSocketError) {}
}

pub struct WebSocketUpgrade<F = DefaultOnFailedUpgrade> {
    config: Config,
    limits: Limits,
    protocol: Option<HeaderValue>,
    /// `None` if HTTP/2+ WebSockets are used.
    sec_websocket_key: Option<HeaderValue>,
    on_upgrade: hyper::upgrade::OnUpgrade,
    on_failed_upgrade: F,
    sec_websocket_protocol: Option<HeaderValue>,
}

impl<F> std::fmt::Debug for WebSocketUpgrade<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketUpgrade")
            .field("config", &self.config)
            .field("protocol", &self.protocol)
            .field("sec_websocket_key", &self.sec_websocket_key)
            .field("sec_websocket_protocol", &self.sec_websocket_protocol)
            .finish_non_exhaustive()
    }
}

impl<S> FromRequestParts<S> for WebSocketUpgrade<DefaultOnFailedUpgrade>
where
    S: Send + Sync,
{
    type Rejection = WebSocketError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let sec_websocket_key = if parts.version <= Version::HTTP_11 || cfg!(not(feature = "http2"))
        {
            if parts.method != Method::GET {
                return Err(WebSocketError::MethodNotGet);
            }

            if !header_contains(&parts.headers, header::CONNECTION, "upgrade") {
                return Err(WebSocketError::InvalidConnectionHeader);
            }

            if !header_eq(&parts.headers, header::UPGRADE, "websocket") {
                return Err(WebSocketError::InvalidUpgradeHeader);
            }

            let sec_websocket_key = parts
                .headers
                .get(header::SEC_WEBSOCKET_KEY)
                .ok_or(WebSocketError::InvalidWebSocketVersionHeader)?
                .clone();

            Some(sec_websocket_key)
        } else {
            if parts.method != Method::CONNECT {
                return Err(WebSocketError::MethodNotConnect);
            }

            // if this feature flag is disabled, we won’t be receiving an HTTP/2 request to begin
            // with.
            #[cfg(feature = "http2")]
            if parts
                .extensions
                .get::<hyper::ext::Protocol>()
                .map_or(true, |p| p.as_str() != "websocket")
            {
                return Err(WebSocketError::InvalidProtocolPseudoheader);
            }

            None
        };

        if !header_eq(&parts.headers, header::SEC_WEBSOCKET_VERSION, "13") {
            return Err(WebSocketError::InvalidWebSocketVersionHeader);
        }

        let on_upgrade = parts
            .extensions
            .remove::<hyper::upgrade::OnUpgrade>()
            .ok_or(WebSocketError::ConnectionNotUpgradeable)?;

        let sec_websocket_protocol = parts.headers.get(header::SEC_WEBSOCKET_PROTOCOL).cloned();

        Ok(Self {
            config: Default::default(),
            limits: Default::default(),
            protocol: None,
            sec_websocket_key,
            on_upgrade,
            sec_websocket_protocol,
            on_failed_upgrade: DefaultOnFailedUpgrade,
        })
    }
}

impl<F> WebSocketUpgrade<F> {
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }

    pub fn on_failed_upgrade<C>(self, callback: C) -> WebSocketUpgrade<C>
    where
        C: OnFailedUpgrade,
    {
        WebSocketUpgrade {
            config: self.config,
            limits: self.limits,
            protocol: self.protocol,
            sec_websocket_key: self.sec_websocket_key,
            on_upgrade: self.on_upgrade,
            on_failed_upgrade: callback,
            sec_websocket_protocol: self.sec_websocket_protocol,
        }
    }

    #[must_use = "to set up the WebSocket connection, this response must be returned"]
    pub fn on_upgrade<C, Fut>(self, callback: C) -> Response
    where
        C: FnOnce(WebSocket) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
        F: OnFailedUpgrade,
    {
        let on_upgrade = self.on_upgrade;
        let config = self.config;
        let limits = self.limits;
        let on_failed_upgrade = self.on_failed_upgrade;

        let protocol = self.protocol.clone();

        tokio::spawn(async move {
            let upgraded = match on_upgrade.await {
                Ok(upgraded) => upgraded,
                Err(err) => {
                    on_failed_upgrade.call(WebSocketError::UpgradeFailed(err));
                    return;
                }
            };
            let upgraded = TokioIo::new(upgraded);

            let socket = tokio_websockets::server::Builder::new()
                .config(config)
                .limits(limits)
                .serve(upgraded);

            let socket = WebSocket::new(socket, protocol);
            callback(socket).await;
        });

        if let Some(sec_websocket_key) = &self.sec_websocket_key {
            // If `sec_websocket_key` was `Some`, we are using HTTP/1.1.

            #[allow(clippy::declare_interior_mutable_const)]
            const UPGRADE: HeaderValue = HeaderValue::from_static("upgrade");
            #[allow(clippy::declare_interior_mutable_const)]
            const WEBSOCKET: HeaderValue = HeaderValue::from_static("websocket");

            let mut builder = Response::builder()
                .status(StatusCode::SWITCHING_PROTOCOLS)
                .header(header::CONNECTION, UPGRADE)
                .header(header::UPGRADE, WEBSOCKET)
                .header(
                    header::SEC_WEBSOCKET_ACCEPT,
                    sign(sec_websocket_key.as_bytes()),
                );

            if let Some(protocol) = self.protocol {
                builder = builder.header(header::SEC_WEBSOCKET_PROTOCOL, protocol);
            }

            builder.body(Body::empty()).unwrap()
        } else {
            // Otherwise, we are HTTP/2+. As established in RFC 9113 section 8.5, we just respond
            // with a 2XX with an empty body:
            // <https://datatracker.ietf.org/doc/html/rfc9113#name-the-connect-method>.
            Response::new(Body::empty())
        }
    }
}

fn sign(key: &[u8]) -> HeaderValue {
    use base64::engine::Engine as _;

    let mut sha1 = sha1::Sha1::default();
    sha1.update(key);
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Sec-WebSocket-Accept
    sha1.update(&b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"[..]);
    let b64 = bytes::Bytes::from(base64::engine::general_purpose::STANDARD.encode(sha1.finalize()));
    HeaderValue::from_maybe_shared(b64).expect("base64 is a valid value")
}

fn header_contains(headers: &HeaderMap, key: HeaderName, value: &'static str) -> bool {
    let header = if let Some(header) = headers.get(&key) {
        header
    } else {
        return false;
    };

    if let Ok(header) = std::str::from_utf8(header.as_bytes()) {
        header.to_ascii_lowercase().contains(value)
    } else {
        false
    }
}

fn header_eq(headers: &HeaderMap, key: HeaderName, value: &'static str) -> bool {
    if let Some(header) = headers.get(&key) {
        header.as_bytes().eq_ignore_ascii_case(value.as_bytes())
    } else {
        false
    }
}
