use std::net::SocketAddr;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};

use self::tcp::TcpOutBound;
use self::udp::UdpOutBound;
use crate::header::Cmd;

pub mod tcp;
pub mod udp;

pub enum OutBound {
    Tcp(TcpOutBound),
    Udp(UdpOutBound),
}

impl OutBound {
    pub async fn new(addr: &SocketAddr, cmd: Cmd) -> anyhow::Result<Self> {
        Ok(match cmd {
            Cmd::Tcp => Self::Tcp(TcpOutBound::new(addr).await?),
            Cmd::Udp => Self::Udp(UdpOutBound::new(addr).await?),
        })
    }
}

impl AsyncRead for OutBound {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Udp(ref mut socket) => Pin::new(socket).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for OutBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Udp(ref mut socket) => Pin::new(socket).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_flush(cx),
            Self::Udp(ref mut socket) => Pin::new(socket).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Udp(ref mut socket) => Pin::new(socket).poll_shutdown(cx),
        }
    }
}
