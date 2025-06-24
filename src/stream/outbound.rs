use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream, UdpSocket};

use anyhow::Context;

pub trait OutBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>>;

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()>;
}

pub struct TcpOutBound {
    stream: TcpStream,
}

pub struct UdpOutBound {
    socket: UdpSocket,
}

impl TcpOutBound {
    pub fn new(addr: &SocketAddr) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).context("failed to stream connect")?;

        stream
            .set_nonblocking(true)
            .context("failed to set nonblocking")?;

        Ok(Self { stream })
    }
}

impl UdpOutBound {
    pub fn new(addr: &SocketAddr) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").context("failed to bind socket")?;

        socket.connect(addr).context("failed to socket connect")?;
        socket
            .set_nonblocking(true)
            .context("failed to set nonblocking")?;

        Ok(Self { socket })
    }
}

impl OutBound for TcpOutBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>> {
        match self.stream.read(buf) {
            Ok(readed) => Ok(Some(readed)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => anyhow::bail!("failed to read stream: {err:?}"),
        }
    }

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        self.stream
            .write_all(buf)
            .context("failed to write to stream")
    }
}

impl OutBound for UdpOutBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>> {
        match self.socket.recv(buf) {
            Ok(readed) => Ok(Some(readed)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => anyhow::bail!("failed to recv from socket: {err:?}"),
        }
    }

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        self.socket.send(buf).context("failed to send to socket")?;

        Ok(())
    }
}
