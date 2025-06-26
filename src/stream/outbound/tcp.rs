use anyhow::Context;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{self, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;

pub struct TcpOutBound {
    stream: TcpStream,
}

impl TcpOutBound {
    pub async fn new(addr: &SocketAddr) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr)
            .await
            .context("failed to stream connect")?;

        Ok(Self { stream })
    }
}

impl AsyncRead for TcpOutBound {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpOutBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}
