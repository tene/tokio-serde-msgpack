extern crate futures;
extern crate tokio;
extern crate tokio_serde_msgpack;

#[macro_use]
extern crate serde_derive;
extern crate rmp_serde;
extern crate serde;

use futures::{future, SinkExt, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

use tokio_serde_msgpack::{from_io, MsgPackReader, MsgPackWriter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
struct Hello {
    id: u32,
    name: String,
}

unsafe impl Send for Hello {}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:17653").await?;

    println!("listening on {:?}", listener.local_addr().unwrap());

    listener
        .incoming()
        .for_each(|result| {
            let socket = result.unwrap();
            println!("New client connection!");
            let (deserialized, serialized): (
                MsgPackReader<TcpStream, Hello>,
                MsgPackWriter<TcpStream, Hello>,
            ) = from_io(socket);

            let deserialized = deserialized.map_err(|e| println!("ERR: {:#?}", e));
            let serialized = serialized.sink_map_err(|e| println!("ERR: {:#?}", e));

            // Spawn a task that prints all received messages to STDOUT
            let handle_conn = deserialized
                .map(move |result| {
                    let msg = result.unwrap();
                    println!("RX: {:#?}", msg);
                    Ok(Hello {
                        id: msg.id,
                        name: format!("server: {}", msg.name),
                    })
                })
                .forward(serialized);

            tokio::spawn(handle_conn);
            future::ready(())
        })
        .await;

    Ok(())
}
