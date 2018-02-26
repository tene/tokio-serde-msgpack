extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_serde_msgpack;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{Sink, Stream, Future};
use futures::sync::mpsc::unbounded;

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use tokio_io::codec::length_delimited;
use tokio_io::AsyncRead;

use tokio_serde_msgpack::{WriteMsgPack, ReadMsgPack};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

pub fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind a server socket
    let listener = TcpListener::bind(
        &"127.0.0.1:17653".parse().unwrap(),
        &handle).unwrap();

    println!("listening on {:?}", listener.local_addr());

    core.run(listener.incoming().for_each(|(socket, _)| {
        let (socket_read, socket_write) = socket.split();
        let (tx, rx) = unbounded::<Hello>();

        let delimited_write = length_delimited::FramedWrite::new(socket_write);
        let serialized = WriteMsgPack::<_, Hello>::new(delimited_write)
            .sink_map_err(|e| {println!("WRITE ERR: {:?}", e); ()});

        let delimited_read = length_delimited::FramedRead::new(socket_read);
        let deserialized = ReadMsgPack::<_, Hello>::new(delimited_read)
            .map_err(|e| println!("READ ERR: {:?}", e));
        
        handle.spawn(rx.forward(serialized).and_then(|(_,mut s)| {Ok(())}));

        let hi = Hello{
            id: 137,
            name: "Server".to_owned(),
        };
        tx.unbounded_send(hi);

        // Spawn a task that prints all received messages to STDOUT
        handle.spawn(deserialized.for_each(move |msg| {
            println!("GOT: {:?}", msg);
            tx.unbounded_send(msg);
            Ok(())
        }));

        Ok(())
    })).unwrap();
}