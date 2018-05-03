extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_serde_msgpack;
extern crate tokio_stdin_stdout;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{Future, Sink, Stream};

use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;

use tokio_io::codec::{length_delimited, FramedRead, LinesCodec};
use tokio_io::{AsyncRead, AsyncWrite};

use tokio_serde_msgpack::{ReadMsgPack, WriteMsgPack};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

pub fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // Bind a server socket
    let socket = TcpStream::connect(&"127.0.0.1:17653".parse().unwrap(), &handle);

    core.run(socket.and_then(|socket| {
        let (socket_read, socket_write) = socket.split();

        let delimited_write = length_delimited::FramedWrite::new(socket_write);
        let serialized = WriteMsgPack::new(delimited_write);

        let delimited_read = length_delimited::FramedRead::new(socket_read);
        let deserialized =
            ReadMsgPack::<_, Hello>::new(delimited_read).map_err(|e| println!("ERR: {:?}", e));

        handle.spawn(deserialized.for_each(|msg| {
            println!("GOT: {:?}", msg);
            Ok(())
        }));

        let stdin = tokio_stdin_stdout::stdin(0);
        let lines_in = FramedRead::new(stdin, LinesCodec::new());

        let send_greetings = lines_in
            .map(|l| Hello { id: 42, name: l })
            .forward(serialized);
        send_greetings
    })).unwrap();
}
