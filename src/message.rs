use tokio_websockets::{CloseCode, Payload};

#[derive(Debug)]
pub enum Message {
    Text(Payload),
    Binary(Payload),
    Close(Option<CloseCode>, String),
    Ping(Payload),
    Pong(Payload),
}

impl From<tokio_websockets::Message> for Message {
    fn from(msg: tokio_websockets::Message) -> Self {
        if msg.is_text() {
            Message::Text(msg.into_payload())
        } else if msg.is_binary() {
            Message::Binary(msg.into_payload())
        } else if msg.is_close() {
            let Some((code, reason)) = msg.as_close() else {
                return Message::Close(None, String::new());
            };

            Message::Close(Some(code), reason.to_string())
        } else if msg.is_ping() {
            Message::Ping(msg.into_payload())
        } else if msg.is_pong() {
            Message::Pong(msg.into_payload())
        } else {
            unreachable!()
        }
    }
}

impl From<Message> for tokio_websockets::Message {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(payload) => tokio_websockets::Message::text(payload),
            Message::Binary(payload) => tokio_websockets::Message::binary(payload),
            Message::Close(code, reason) => tokio_websockets::Message::close(code, &reason),
            Message::Ping(payload) => tokio_websockets::Message::ping(payload),
            Message::Pong(payload) => tokio_websockets::Message::pong(payload),
        }
    }
}
