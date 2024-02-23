use std::task::Poll;

use futures_util::{
    sink::{Sink, SinkExt},
    stream::{Stream, StreamExt},
};
use http::HeaderValue;
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio_websockets::{Message, WebSocketStream};

use crate::WebSocketError;

#[derive(Debug)]
pub struct WebSocket {
    inner: WebSocketStream<TokioIo<Upgraded>>,
    pub protocol: Option<HeaderValue>,
}

impl WebSocket {
    pub fn new(inner: WebSocketStream<TokioIo<Upgraded>>, protocol: Option<HeaderValue>) -> Self {
        Self { inner, protocol }
    }

    /// Receive the next message from the connection.
    pub async fn recv(&mut self) -> Option<Result<Message, WebSocketError>> {
        self.next().await
    }

    /// Send a message to the connection.
    pub async fn send(&mut self, msg: Message) -> Result<(), WebSocketError> {
        self.inner.send(msg).await.map_err(|e| e.into())
    }

    /// Close the connection.
    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        self.inner.close().await.map_err(|e| e.into())
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, WebSocketError>;

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
