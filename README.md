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

_Work in progress._

## Example

_Work in progress._

## Contributing

Contributions are always welcome! If you have an idea for a feature or find a
bug, let me know. PR's are appreciated, but if it's not a small change, please
open an issue first so we're all on the same page!

## License

`axum-tws` is dual-licensed under either

- **[MIT License](/LICENSE-MIT)**
- **[Apache License, Version 2.0](/LICENSE-APACHE)**

at your option.
