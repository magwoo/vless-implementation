use anyhow::Context;
use inbound::{tcp::TcpInBound, udp::UdpInBound};
use outbound::{TcpOutBound, UdpOutBound};
use std::net::{SocketAddr, TcpStream};

use self::inbound::InBound;
use self::outbound::OutBound;
use crate::header::{Cmd, Header};

mod inbound;
mod outbound;

pub struct Stream {
    header: Header,
    client_addr: SocketAddr,
    inbound: Box<dyn InBound>,
    outbound: Box<dyn OutBound>,
}

impl Stream {
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn from_incoming(mut stream: TcpStream, client_addr: SocketAddr) -> anyhow::Result<Self> {
        let header = Header::from_reader(&mut stream).context("failed to parse header")?;

        let inbound: Box<dyn InBound> = match header.cmd() {
            Cmd::Tcp => Box::new(TcpInBound::new(stream)),
            Cmd::Udp => Box::new(UdpInBound::new(stream)),
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
            client_addr,
            inbound,
            outbound,
        })
    }

    pub fn event_loop(&mut self) -> anyhow::Result<()> {
        let mut buf = [0; u16::MAX as usize];

        loop {
            if let Some(readed) = self
                .inbound
                .read(&mut buf)
                .context("failed to read from inbound")?
            {
                if readed == 0 {
                    return Ok(());
                }

                println!(
                    "{} | {} -> {}: {} bytes",
                    self.header.cmd(),
                    self.client_addr,
                    self.header.addr(),
                    readed
                );

                self.outbound
                    .write(&buf[..readed])
                    .context("failed to write to outbound")?;
            }

            if let Some(readed) = self
                .outbound
                .read(&mut buf)
                .context("failed to read from outbound")?
            {
                if readed == 0 {
                    return Ok(());
                }

                println!(
                    "{} | {} <- {}: {} bytes",
                    self.header.cmd(),
                    self.client_addr,
                    self.header.addr(),
                    readed
                );

                self.inbound
                    .write(&buf[..readed])
                    .context("failed to write to inbound")?;
            }
        }
    }
}
