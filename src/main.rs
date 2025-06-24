use std::net::TcpListener;

use crate::stream::Stream;

mod header;
mod stream;
mod transport;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();

    while let Ok((stream, addr)) = listener.accept() {
        std::thread::spawn(move || {
            let mut stream = Stream::from_incoming(stream).unwrap();

            println!("handled new stream: {}", addr);

            stream.event_loop().unwrap();
        });
    }
}
