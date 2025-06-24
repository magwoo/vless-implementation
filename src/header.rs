use std::fmt::Display;
use std::io::Read;
use std::net::{SocketAddr, ToSocketAddrs};

use anyhow::Context;

#[derive(Clone, Copy)]
pub enum Cmd {
    Tcp,
    Udp,
}

pub struct Header {
    version: u8,
    uuid: [u8; 16],
    cmd: Cmd,
    addr: SocketAddr,
}

impl Header {
    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn uuid(&self) -> &[u8; 16] {
        &self.uuid
    }

    pub fn cmd(&self) -> Cmd {
        self.cmd
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }

    pub fn from_reader(reader: &mut impl Read) -> anyhow::Result<Self> {
        let mut client_part_buf = [0; 18];
        reader
            .read_exact(&mut client_part_buf)
            .context("failed to read client part")?;

        let version = client_part_buf[0];

        if version != 0 {
            anyhow::bail!("unsupported protocol version: {}", version);
        }

        let uuid = client_part_buf[1..17].try_into().unwrap();

        let opt_len = client_part_buf[17];

        if opt_len != 0 {
            let mut options_buf = vec![0; opt_len as usize];
            reader
                .read_exact(&mut options_buf)
                .context("failed to read options")?;
        }

        let mut addr_part_buf = [0; 4];
        reader
            .read_exact(&mut addr_part_buf)
            .context("failed to read addr part")?;

        let cmd = Cmd::try_from(addr_part_buf[0]).context("failed to parse cmd")?;
        let port = u16::from_be_bytes(addr_part_buf[1..3].try_into().unwrap());
        let addr_type = addr_part_buf[3];

        let addr = match addr_type {
            1 => {
                let mut ip_buf = [0; 4];
                reader
                    .read_exact(&mut ip_buf)
                    .context("failed to read ip addr")?;

                SocketAddr::new(ip_buf.into(), port)
            }
            2 => {
                let mut len_buf = [0; 1];
                reader
                    .read_exact(&mut len_buf)
                    .context("failed to read domain len")?;

                let mut domain_buf = vec![0; len_buf[0] as usize];
                reader
                    .read_exact(&mut domain_buf)
                    .context("failed to read domain")?;

                let domain = String::from_utf8_lossy(&domain_buf);
                let host = format!("{}:{}", domain, port);

                host.to_socket_addrs()
                    .with_context(|| format!("failed to lookup host: {}", host))?
                    .next()
                    .with_context(|| format!("missing any lookup result for host: {}", host))?
            }
            other => anyhow::bail!("unknown addr type: {}", other),
        };

        Ok(Self {
            version,
            uuid,
            cmd,
            addr,
        })
    }
}

impl TryFrom<u8> for Cmd {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Tcp),
            2 => Ok(Self::Udp),
            other => anyhow::bail!("unknown value: {}", other),
        }
    }
}

impl Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tcp => write!(f, "TCP"),
            Self::Udp => write!(f, "UDP"),
        }
    }
}
