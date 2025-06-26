use anyhow::Context;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{self, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UdpSocket;

pub struct UdpOutBound {
    socket: UdpSocket,
}

impl UdpOutBound {
    pub async fn new(addr: &SocketAddr) -> anyhow::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .context("failed to bind socket")?;

        socket
            .connect(addr)
            .await
            .with_context(|| format!("failed to connect to: {}", addr))?;

        Ok(Self { socket })
    }
}

impl AsyncRead for UdpOutBound {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.socket).poll_recv(cx, buf)
    }
}

impl AsyncWrite for UdpOutBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.socket).poll_send(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }
}
