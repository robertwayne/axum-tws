# axum-tws

<div align="right">
<a href="https://crates.io/crates/axum-tws">
    <img src="https://img.shields.io/crates/v/axum-tws?style=flat-square" alt="crates.io badge">
</a>
<a href="https://docs.rs/axum-tws/latest/">
    <img src="https://img.shields.io/docsrs/axum-tws?style=flat-square" alt="docs.rs badge">
</a>
</div>
<br>

`axum-tws` is an alternative WebSocket extractor for
__[axum](https://github.com/tokio-rs/axum)__ using
__[tokio-websockets](https://github.com/Gelbpunkt/tokio-websockets/)__ as the
underlying WebSocket library instead of `tungstenite`.

It is not a complete drop-in replacement and has no intention to be one. While
your upgrade handler will look the same, working with `Message` types in
`tokio-websockets` is slightly different from `tungstenite`. Please refer to the
__[tokio-websockets
documentation](https://docs.rs/tokio-websockets/latest/tokio_websockets/)__ for
detailed information, or take a look at the example below.

Much of the code has been ported directly from the __[axum::extract::ws
module](https://docs.rs/axum/latest/axum/extract/ws/index.html)__ - all credit
goes to the original authors.

## Getting Started

Run `cargo add axum-tws` to add the library to your project.

## Echo Server Example

If you have cloned the `axum-tws` repository, you can run the `echo_server` example with the
command `cargo run --example echo_server`. You can then connect to it with `wscat` or similar on
`127.0.0.1:3000/ws`.

```rust no_run
use axum::{response::Response, routing::get, Router};
use axum_tws::{WebSocket, WebSocketUpgrade};

#[tokio::main(flavor = "current_thread")]
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
        if msg.is_text() {
            socket.send(msg).await?;
        }
    }

    Ok(())
}
```

## Feature Flags

| Flag    | Default  | Description                             |
|---------|----------|-----------------------------------------|
| `http2` | Disabled | Adds support for WebSockets over HTTP/2 |

## Contributing

Contributions are always welcome! If you have an idea for a feature or find a
bug, let me know. PR's are appreciated, but if it's not a small change, please
open an issue first so we're all on the same page!

## License

`axum-tws` is dual-licensed under either

- **[MIT License](/LICENSE-MIT)**
- **[Apache License, Version 2.0](/LICENSE-APACHE)**

at your option.
