extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_serde_msgpack;
extern crate tokio_stdin_stdout;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{Future, Sink, Stream};
use tokio::net::TcpStream;
use tokio_io::codec::{FramedRead, LinesCodec};

use tokio_serde_msgpack::{from_io, MsgPackReader, MsgPackWriter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

pub fn main() {
    let blah = TcpStream::connect(&"127.0.0.1:17653".parse().unwrap());

    let client = blah.map(move |socket| {
        println!("Connected!");
        let (deserialized_err, serialized): (
            MsgPackReader<TcpStream, Hello>,
            MsgPackWriter<TcpStream, Hello>,
        ) = from_io(socket);
        let deserialized = deserialized_err.map_err(|e| println!("ERR: {:#?}", e));

        tokio::spawn(
            deserialized
                .for_each(|msg| {
                    println!("GOT: {:?}", msg);
                    Ok(())
                })
                .map_err(|e| println!("ERR: {:#?}", e)),
        );

        let stdin = tokio_stdin_stdout::stdin(0);
        let lines_in = FramedRead::new(stdin, LinesCodec::new());

        let mut counter = 0;
        let greetings = lines_in.map(move |l| {
            counter = counter + 1;
            Hello { id: counter, name: l }
        });

        let send_greetings = serialized.send_all(greetings).then(|_| Ok(()));
        tokio::spawn(send_greetings);
    }).map_err(|e| println!("ERR: {:#?}", e));
    tokio::run(client);
}
