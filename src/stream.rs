use anyhow::Context;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use self::inbound::InBound;
use self::outbound::OutBound;
use crate::header::Header;

mod inbound;
mod io;
mod outbound;

pub struct Stream {
    header: Header,
    client_addr: SocketAddr,
    inbound: InBound,
    outbound: OutBound,
}

impl Stream {
    const BUFFER_SIZE: usize = u16::MAX as usize;

    pub async fn from_incoming(
        mut stream: TcpStream,
        client_addr: SocketAddr,
    ) -> anyhow::Result<Self> {
        let header = Header::from_reader(&mut stream)
            .await
            .context("failed to parse header")?;

        let inbound = InBound::new(stream, header.cmd());

        let addr = header.addr();
        let outbound = OutBound::new(addr, header.cmd())
            .await
            .context("failed to create outbound")?;

        Ok(Self {
            header,
            client_addr,
            inbound,
            outbound,
        })
    }

    pub async fn event_loop(self) -> anyhow::Result<()> {
        let Self {
            inbound,
            outbound,
            header,
            client_addr,
        } = self;

        let (mut inbound_reader, mut inbound_writer) = tokio::io::split(inbound);
        let (mut outbound_reader, mut outbound_writer) = tokio::io::split(outbound);

        let header_clone = header.clone();

        let in_out = tokio::spawn(async move {
            let mut buf = [0; Self::BUFFER_SIZE];

            loop {
                let readed = inbound_reader
                    .read(&mut buf)
                    .await
                    .context("failed to read from inbound")?;

                if readed == 0 {
                    break;
                }

                let writed = outbound_writer
                    .write(&buf[..readed])
                    .await
                    .context("failed to write to outbound")?;

                println!(
                    "{} | {} -> {}: {} bytes",
                    header_clone.cmd(),
                    client_addr,
                    header_clone.addr(),
                    readed
                );

                if writed != readed {
                    panic!("writed != readed! writed: {}, readed: {}", writed, readed)
                }
            }

            anyhow::Ok(())
        });

        let out_in = tokio::spawn(async move {
            let mut buf = [0; Self::BUFFER_SIZE];

            loop {
                let readed = outbound_reader
                    .read(&mut buf)
                    .await
                    .context("failed to read from inbound")?;

                if readed == 0 {
                    break;
                }

                println!(
                    "{} | {} <- {}: {} bytes",
                    header.cmd(),
                    client_addr,
                    header.addr(),
                    readed
                );

                let writed = inbound_writer
                    .write(&buf[..readed])
                    .await
                    .context("failed to write to outbound")?;

                if writed != readed {
                    panic!("writed != readed! writed: {}, readed: {}", writed, readed)
                }
            }

            anyhow::Ok(())
        });

        let (in_out, out_in) = tokio::join!(in_out, out_in);

        in_out.context("in_out failed")?.context("in_out failed")?;
        out_in.context("out_in failed")?.context("out_in failed")?;

        Ok(())

        // loop {
        // let readed = inbound
        //     .read(&mut buf)
        //     .await
        //     .context("failed to read from inbound")?;

        // // println!("to net: {:?}", &buf[..readed]);

        // if readed == 0 {
        //     return Ok(());
        // }

        // outbound
        //     .write(&buf[..readed])
        //     .await
        //     .context("failed to write to outbound")?;

        // println!(
        //     "{} | {} -> {}: {} bytes",
        //     header.cmd(),
        //     client_addr,
        //     header.addr(),
        //     readed
        // );

        // let readed = outbound
        //     .read(&mut buf)
        //     .await
        //     .context("failed to read from outbound")?;

        // // println!("from net: {:?}", &buf[..readed]);

        // if readed == 0 {
        //     return Ok(());
        // }

        // inbound
        //     .write(&buf[..readed])
        //     .await
        //     .context("failed to write to inbound")?;

        // println!(
        //     "{} | {} <- {}: {} bytes",
        //     header.cmd(),
        //     client_addr,
        //     header.addr(),
        //     readed
        // );
        // }

        // let mut inbound = IOBoundWrapper::new(inbound);
        // let mut outbound = IOBoundWrapper::new(outbound);

        // let (outbound_readed, inbound_readed) =
        //     tokio::io::copy_bidirectional(&mut inbound, &mut outbound)
        //         .await
        //         .context("failed to io bidirectional")?;

        // println!(
        //     "stream closed with outbound: {}, inbound: {}",
        //     outbound_readed, inbound_readed
        // );

        // Ok(())
    }
}
