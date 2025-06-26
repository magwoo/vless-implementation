use std::pin::Pin;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

pub struct TcpInBound {
    stream: TcpStream,
    is_first: bool,
}

impl AsyncRead for TcpInBound {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpInBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        if self.is_first {
            let mut write_buf = Vec::with_capacity(buf.len() + 2);
            write_buf.extend(&[0, 0]);
            write_buf.extend(buf);

            match Pin::new(&mut self.stream).poll_write(cx, &write_buf) {
                Poll::Ready(Ok(writed)) => {
                    self.is_first = false;
                    Poll::Ready(Ok(writed - 2))
                }
                Poll::Ready(Err(err)) => Poll::Ready(Err(std::io::Error::other(err))),
                Poll::Pending => Poll::Pending,
            }
        } else {
            Pin::new(&mut self.stream).poll_write(cx, buf)
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl TcpInBound {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            is_first: true,
        }
    }
}
