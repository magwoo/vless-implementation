use anyhow::Context;
use tokio::net::TcpListener;

use crate::stream::Stream;

mod header;
mod stream;
mod transport;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:80")
        .await
        .context("failed to bind listener")?;

    loop {
        let (stream, addr) = listener
            .accept()
            .await
            .context("failed to accept a connection")?;

        println!("handled new stream: {}", addr);

        tokio::spawn(async move {
            let stream = match Stream::from_incoming(stream, addr).await {
                Ok(stream) => stream,
                Err(err) => {
                    println!("failed to prepare stream with {}: {:?}", addr, err);
                    return;
                }
            };

            stream
                .event_loop()
                .await
                .map_err(|err| println!("stream error with {}: {:?}", addr, err))
                .ok();

            println!("stream closed: {}", addr)
        });
    }

    // while let Ok((stream, addr)) = listener.accept().await {
    //     std::thread::spawn(move || {
    //         let mut stream = Stream::from_incoming(stream, addr).unwrap();

    //         println!("handled new stream: {}", addr);

    //         stream.event_loop().unwrap();
    //     });
    // }
}
