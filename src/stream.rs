use std::io::BufReader;
use std::net::TcpStream;

use anyhow::Context;
use inbound::{tcp::TcpInBound, udp::UdpInBound};
use outbound::{TcpOutBound, UdpOutBound};

use self::inbound::InBound;
use self::outbound::OutBound;
use crate::header::{Cmd, Header};

mod inbound;
mod outbound;

pub struct Stream {
    header: Header,
    inbound: Box<dyn InBound>,
    outbound: Box<dyn OutBound>,
}

impl Stream {
    pub fn from_incoming(stream: TcpStream) -> anyhow::Result<Self> {
        let mut reader = BufReader::new(stream);

        let header = Header::from_reader(&mut reader).context("failed to parse header")?;

        let inbound: Box<dyn InBound> = match header.cmd() {
            Cmd::Tcp => Box::new(TcpInBound::new(reader.into_inner())),
            Cmd::Udp => Box::new(UdpInBound::new(reader.into_inner())),
        };

        let addr = header.addr();
        let outbound: Box<dyn OutBound> = match header.cmd() {
            Cmd::Tcp => Box::new(
                TcpOutBound::new(addr)
                    .with_context(|| format!("failed to create outbound to: {}", addr))?,
            ),
            Cmd::Udp => Box::new(
                UdpOutBound::new(addr)
                    .with_context(|| format!("failed to create outbound to: {}", addr))?,
            ),
        };

        Ok(Self {
            header,
            inbound,
            outbound,
        })
    }

    pub fn process() {
        unimplemented!()
    }
}
