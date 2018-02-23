extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_serde_msgpack;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{Future, Sink};

use tokio_core::reactor::Core;
use tokio_core::net::TcpStream;

use serde::{Deserialize, Serialize};

// Use length delimited frames
use tokio_io::codec::length_delimited;

use tokio_serde_msgpack::WriteMsgPack;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

pub fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind a server socket
    let socket = TcpStream::connect(
        &"127.0.0.1:17653".parse().unwrap(),
        &handle);

    core.run(socket.and_then(|socket| {

        // Delimit frames using a length header
        let length_delimited = length_delimited::FramedWrite::new(socket);

        // Serialize frames with MsgPack
        let serialized = WriteMsgPack::new(length_delimited);

        let hi = Hello {
            id: 42,
            name: "Client Forty Two".to_owned(),
        };

        // Send the value
        serialized.send(hi)
    })).unwrap();
}