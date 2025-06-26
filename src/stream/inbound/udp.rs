use std::collections::VecDeque;
use std::io::{BufRead, Read};
use std::pin::Pin;
use std::task::Poll;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;

pub struct UdpInBound {
    stream: TcpStream,
    buffer: VecDeque<u8>,
    read_buf: [u8; 1024],
    write_buf: Vec<u8>,
    is_first: bool,
}

impl UdpInBound {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: VecDeque::default(),
            read_buf: [0; 1024],
            write_buf: Vec::default(),
            is_first: true,
        }
    }
}

impl AsyncRead for UdpInBound {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let Self {
            stream,
            buffer,
            read_buf,
            ..
        } = &mut *self;

        let mut read_buf = ReadBuf::new(read_buf);

        match Pin::new(stream).poll_read(cx, &mut read_buf) {
            Poll::Pending => {}
            Poll::Ready(Ok(())) => {
                buffer.extend(read_buf.filled());
            }
            Poll::Ready(Err(err)) => return Poll::Ready(Err(std::io::Error::other(err))),
        }

        if self.buffer.len() < 2 {
            return Poll::Pending;
        }

        let mut len_bytes = self.buffer.iter().take(2);
        let data_len =
            u16::from_be_bytes([*len_bytes.next().unwrap(), *len_bytes.next().unwrap()]) as usize;

        if self.buffer.len() < data_len + 2 {
            return Poll::Pending;
        }

        self.buffer.consume(2);

        let read_buf = &mut buf.initialize_unfilled()[..data_len];

        self.buffer.read_exact(read_buf)?;

        buf.advance(data_len);

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for UdpInBound {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.write_buf.clear();

        let Self {
            stream,
            write_buf,
            is_first,
            ..
        } = &mut *self;

        if *is_first {
            write_buf.extend(&[0, 0]);
        }

        write_buf.extend((buf.len() as u16).to_be_bytes());
        write_buf.extend(buf);

        match Pin::new(stream).poll_write(cx, write_buf) {
            Poll::Ready(Ok(mut writed)) => {
                if self.is_first {
                    writed -= 2;
                    self.is_first = false;
                }
                Poll::Ready(Ok(writed - 2))
            }
            Poll::Ready(Err(err)) => Poll::Ready(Err(std::io::Error::other(err))),
            Poll::Pending => Poll::Pending,
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
