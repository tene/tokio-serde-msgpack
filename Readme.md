# Tokio / Serde bindings for MsgPack

Utilities needed to easily implement a Tokio MsgPack transport using [serde] for
MsgPack serialization and deserialization of frame values.

## Usage

To use `tokio-serde-msgpack`, first add this to your `Cargo.toml`:

```toml
[dependencies]
tokio-serde-msgpack = { git = "https://github.com/tene/tokio-serde-msgpack" }
```

Next, add this to your crate:

```rust
extern crate tokio_serde_msgpack;

use tokio_serde_msgpack::{ReadMsgPack, WriteMsgPack};
```

[serde]: https://serde.rs

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in sokio-serde-msgpack by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.