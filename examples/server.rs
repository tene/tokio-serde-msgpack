extern crate futures;
extern crate tokio;
extern crate tokio_serde_msgpack;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{Future, Sink, Stream};
use tokio::net::{TcpListener, TcpStream};

use tokio_serde_msgpack::{from_io, MsgPackReader, MsgPackWriter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

unsafe impl Send for Hello {}

pub fn main() {
    let listener = TcpListener::bind(&"127.0.0.1:17653".parse().unwrap()).unwrap();

    println!("listening on {:?}", listener.local_addr());

    let server = listener
        .incoming()
        .for_each(|socket| {
            println!("New client connection!");
            let (deserialized, serialized): (
                MsgPackReader<TcpStream, Hello>,
                MsgPackWriter<TcpStream, Hello>,
            ) = from_io(socket);

            let deserialized = deserialized.map_err(|e| println!("ERR: {:#?}", e));
            let serialized = serialized.sink_map_err(|e| println!("ERR: {:#?}", e));

            // Spawn a task that prints all received messages to STDOUT
            let handle_conn = deserialized
                .map(move |msg| {
                    println!("RX: {:#?}", msg);
                    Hello {
                        id: msg.id,
                        name: format!("server: {}", msg.name),
                    }
                })
                .forward(serialized)
                .then(|_| Ok(()));

            tokio::spawn(handle_conn);

            Ok(())
        })
        .map_err(|e| println!("ERR: {:#?}", e));

    tokio::run(server);
}
