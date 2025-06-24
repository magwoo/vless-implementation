use anyhow::Context;
use std::collections::VecDeque;
use std::io::{BufRead, ErrorKind, Read, Write};
use std::net::TcpStream;

use super::InBound;

const MAX_BUFFER_SIZE: usize = u16::MAX as usize;

pub struct UdpInBound {
    stream: TcpStream,
    buffer: VecDeque<u8>,
}

impl UdpInBound {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            stream,
            buffer: VecDeque::default(),
        }
    }
}

impl InBound for UdpInBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>> {
        let mut stream_buf = [0; 2048];

        let readed = match self.stream.read(&mut stream_buf) {
            Ok(readed) => readed,
            Err(err) if err.kind() == ErrorKind::WouldBlock => 0,
            Err(err) => anyhow::bail!("failed to read from stream: {err:?}"),
        };

        if readed > 0 {
            if self.buffer.len() > MAX_BUFFER_SIZE {
                self.buffer.clear();
                anyhow::bail!("Buffer overflow");
            }

            self.buffer.extend(&stream_buf[..readed]);
        }

        if self.buffer.len() <= 2 {
            return Ok(None);
        }

        let mut len_bytes = self.buffer.iter().take(2);
        let data_len =
            u16::from_be_bytes([*len_bytes.next().unwrap(), *len_bytes.next().unwrap()]) as usize;

        if self.buffer.len() < data_len + 2 {
            return Ok(None);
        }

        self.buffer.consume(2);

        let data_buf = &mut buf[..data_len];

        self.buffer
            .read_exact(data_buf)
            .context("failed to read data from buffer")?;

        Ok(Some(data_len))
    }

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        let len_bytes = (buf.len() as u16).to_be_bytes();

        let mut buf_with_len = Vec::with_capacity(buf.len() + len_bytes.len());
        buf_with_len.extend_from_slice(&len_bytes);
        buf_with_len.extend_from_slice(buf);

        self.stream
            .write_all(&buf_with_len)
            .context("failed to write to stream")?;

        Ok(())
    }
}
