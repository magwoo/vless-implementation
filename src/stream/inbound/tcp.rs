use anyhow::Context;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

use super::InBound;

pub struct TcpInBound {
    stream: TcpStream,
    is_first: bool,
}

impl TcpInBound {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();

        Self {
            stream,
            is_first: true,
        }
    }
}

impl InBound for TcpInBound {
    fn read(&mut self, buf: &mut [u8]) -> anyhow::Result<Option<usize>> {
        match self.stream.read(buf) {
            Ok(readed) => Ok(Some(readed)),
            Err(err) if err.kind() == ErrorKind::WouldBlock => Ok(None),
            Err(err) => anyhow::bail!("failed to read stream: {err:?}"),
        }
    }

    fn write(&mut self, buf: &[u8]) -> anyhow::Result<()> {
        if self.is_first {
            let mut new_buf = Vec::with_capacity(buf.len() + 2);
            new_buf.extend_from_slice(&[0, 0]);
            new_buf.extend_from_slice(buf);

            self.is_first = false;

            self.stream.write_all(&new_buf)
        } else {
            self.stream.write_all(buf)
        }
        .context("failed to write to stream")
    }
}
