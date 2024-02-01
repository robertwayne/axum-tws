# axum-tws

`axum-tws` is an alternative WebSocket extractor for
__[axum](https://github.com/tokio-rs/axum)__ using
__[tokio-websockets](https://github.com/Gelbpunkt/tokio-websockets/)__ as the
underlying WebSocket library instead of `tungstenite`.

It is mostly a drop-in replacement, with the exception that `Message`'s take the
`tokio_websockets::proto::Payload` type, which is a wrapper around
`bytes::Bytes`, instead of the `Vec<u8>` that `tungstenite` uses.

Much of the code has been ported directly from the __[axum::extract::ws
module](https://docs.rs/axum/latest/axum/extract/ws/index.html)__ - all credit
goes to the original authors.

_This library is currently a work in progress. I wouldn't necessarily recommend
using it, but it is functional. I cannot guarantee API stability, though._

## Getting Started

Run `cargo add --git https://github.com/robertwayne/axum-tws` to add the library
to your project.

## Echo Server Example

```rust
// [dependencies]
// axum = "0.7"
// axum-tws = { git = "https://github.com/robertwayne/axum-tws" }
// tokio = { version = "1", features = ["macros", "rt-multi-thread"] }

use axum::{response::Response, routing::get, Router};
use axum_tws::{Message, WebSocket, WebSocketUpgrade};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;

    axum::serve(listener, Router::new().route("/ws", get(handle_upgrade))).await?;

    Ok(())
}

async fn handle_upgrade(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade({
        move |socket| async {
            if let Err(e) = handle_ws(socket).await {
                println!("websocket error: {:?}", e);
            }
        }
    })
}

async fn handle_ws(mut socket: WebSocket) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            println!("received: {}", String::from_utf8_lossy(&text));
            socket.send(Message::Text(text)).await?;
        }
    }

    Ok(())
}
```

## Contributing

Contributions are always welcome! If you have an idea for a feature or find a
bug, let me know. PR's are appreciated, but if it's not a small change, please
open an issue first so we're all on the same page!

## License

`axum-tws` is dual-licensed under either

- **[MIT License](/LICENSE-MIT)**
- **[Apache License, Version 2.0](/LICENSE-APACHE)**

at your option.
