use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

use self::tcp::TcpInBound;
use self::udp::UdpInBound;
use crate::header::Cmd;

pub mod tcp;
pub mod udp;

pub enum InBound {
    Tcp(TcpInBound),
    Udp(UdpInBound),
}

impl InBound {
    pub fn new(stream: TcpStream, cmd: Cmd) -> Self {
        match cmd {
            Cmd::Tcp => Self::Tcp(TcpInBound::new(stream)),
            Cmd::Udp => Self::Udp(UdpInBound::new(stream)),
        }
    }
}

impl AsyncRead for InBound {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
            Self::Udp(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for InBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
            Self::Udp(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_flush(cx),
            Self::Udp(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match *self {
            Self::Tcp(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
            Self::Udp(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}
