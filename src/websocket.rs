use std::task::Poll;

use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use http::HeaderValue;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio_websockets::WebSocketStream;

use crate::{Message, WebSocketError};

#[derive(Debug)]
pub struct WebSocket {
    inner: WebSocketStream<TokioIo<Upgraded>>,
    pub protocol: Option<HeaderValue>,
}

impl WebSocket {
    pub fn new(inner: WebSocketStream<TokioIo<Upgraded>>, protocol: Option<HeaderValue>) -> Self {
        Self { inner, protocol }
    }

    pub async fn recv(&mut self) -> Option<Result<Message, WebSocketError>> {
        let msg = self.next().await;

        if let Some(Ok(msg)) = msg {
            Some(Ok(msg.into()))
        } else if let Some(Err(e)) = msg {
            Some(Err(e))
        } else {
            None
        }
    }

    pub async fn send(&mut self, msg: Message) -> Result<(), WebSocketError> {
        self.inner.send(msg.into()).await.map_err(|e| e.into())
    }

    /// Gracefully close this WebSocket.
    pub async fn close(mut self) -> Result<(), WebSocketError> {
        self.inner.close().await.map_err(|e| e.into())
    }
}

impl Stream for WebSocket {
    type Item = Result<tokio_websockets::Message, WebSocketError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner
            .poll_next_unpin(cx)
            .map(|opt| opt.map(|res| res.map_err(|e| e.into())))
    }
}

impl Sink<tokio_websockets::Message> for WebSocket {
    type Error = WebSocketError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready_unpin(cx).map_err(|e| e.into())
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: tokio_websockets::Message,
    ) -> Result<(), Self::Error> {
        self.inner.start_send_unpin(item).map_err(|e| e.into())
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_flush_unpin(cx).map_err(|e| e.into())
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_close_unpin(cx).map_err(|e| e.into())
    }
}
