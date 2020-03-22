extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_msgpack;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{future, SinkExt, StreamExt, TryStreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, LinesCodec};

use tokio_serde_msgpack::{from_io, MsgPackReader, MsgPackWriter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

#[tokio::main]
async fn main() {
    let socket = TcpStream::connect("127.0.0.1:17653").await.unwrap();

    println!("Connected!");
    let (deserialized_err, mut serialized): (
        MsgPackReader<TcpStream, Hello>,
        MsgPackWriter<TcpStream, Hello>,
    ) = from_io(socket);
    let deserialized = deserialized_err.map_err(|e| println!("ERR: {:#?}", e));

    tokio::spawn(deserialized.for_each(|msg| {
        println!("GOT: {:?}", msg);
        future::ready(())
    }));

    let lines_in = FramedRead::new(tokio::io::stdin(), LinesCodec::new());

    let mut counter = 0;
    let mut greetings = lines_in.map(move |result| {
        let line = result.unwrap();
        counter = counter + 1;
        Ok(Hello {
            id: counter,
            name: line,
        })
    });

    let send_greetings = serialized.send_all(&mut greetings);
    send_greetings.await.unwrap();
}
