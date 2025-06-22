use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};

mod header;
mod stream;
mod transport;

pub enum OutBound {
    Udp(UdpSocket),
    Tcp(TcpStream),
}

fn handle(mut stream: TcpStream, client_addr: SocketAddr) {
    let mut buf = [0; 65000];
    let mut is_first = true;

    let readed = stream.read(&mut buf).unwrap();

    let version = buf[0];
    let _uuid = &buf[1..17];
    let extra_len = buf[17] as usize;

    if extra_len != 0 {
        panic!("extra len > 0: {}", extra_len);
    }

    let instruction = buf[18 + extra_len];
    let port = u16::from_be_bytes(buf[19 + extra_len..21 + extra_len].try_into().unwrap());
    let addr_type = buf[21];

    let (addr_len, addr) = match addr_type {
        1 => (
            4,
            SocketAddr::new(
                Ipv4Addr::new(
                    buf[22 + extra_len],
                    buf[23 + extra_len],
                    buf[24 + extra_len],
                    buf[25 + extra_len],
                )
                .into(),
                port,
            ),
        ),
        2 => {
            let len = buf[22 + extra_len] as usize;
            let domain =
                String::from_utf8(buf[23 + extra_len..23 + extra_len + len].to_vec()).unwrap();
            println!("lookup domain: {}", domain);
            let domain = format!("{domain}:{port}");
            (1 + len, domain.to_socket_addrs().unwrap().next().unwrap())
        }
        t => panic!("unknown addr type: {t}, buf: {:?}", &buf[..readed]),
    };
    let payload = &buf[22 + extra_len + addr_len..readed];

    println!(
        "-- new connection:\nversion: {}, instruction: {}, addr: {}, initial payload len: {}",
        version,
        instruction,
        addr,
        payload.len(),
    );

    let mut outbound = OutBound::from_instruct(instruction, addr);

    if !payload.is_empty() {
        outbound.write(payload);
        println!(
            "\n -- to net({}): {} -> {}: {:?}",
            payload.len(),
            client_addr,
            addr,
            String::from_utf8_lossy(payload)
        );
    }

    stream.set_nonblocking(true).unwrap();

    loop {
        let mut buf = [0; 65000];

        if let Ok(readed) = stream.read(&mut buf) {
            if readed == 0 {
                println!("tunnel closed by client: {} - {}", client_addr, addr);
                drop(stream);
                drop(outbound);
                return;
            }

            outbound.write(&buf[..readed]);
            println!(
                "\n-- to net({}): {} -> {}: {:?}",
                readed,
                client_addr,
                addr,
                String::from_utf8_lossy(&buf[..readed])
            );
        }

        if let Some(readed) = outbound.read(&mut buf) {
            if readed == 0 {
                println!("tunnel closed by endpoint: {} - {}", client_addr, addr);
                drop(stream);
                drop(outbound);
                return;
            }

            if is_first {
                let mut response = vec![0, 0];
                response.extend_from_slice(&buf[..readed]);

                stream.write_all(&response).unwrap();

                is_first = false;

                println!(
                    "\n -- first from net({}): {} -> {}: {:?}",
                    readed,
                    addr,
                    client_addr,
                    String::from_utf8_lossy(&response)
                );
            } else {
                stream.write_all(&buf[..readed]).unwrap();

                println!(
                    "\n -- from net({}): {} -> {}: {:?}",
                    readed,
                    addr,
                    client_addr,
                    String::from_utf8_lossy(&buf[..readed])
                );
            }
        }

        std::thread::yield_now();
    }
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();

    while let Ok((stream, addr)) = listener.accept() {
        std::thread::spawn(move || {
            handle(stream, addr);
        });
    }
}

impl OutBound {
    pub fn from_instruct(instruct: u8, addr: SocketAddr) -> Self {
        println!("connecting to {}", addr);
        let result = match instruct {
            1 => {
                let socket = TcpStream::connect(addr).unwrap();
                socket.set_nonblocking(true).unwrap();
                Self::Tcp(socket)
            }
            2 => {
                let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
                socket.set_nonblocking(true).unwrap();
                socket.connect(addr).unwrap();
                Self::Udp(socket)
            }
            i => panic!("unknown instruct: {i}"),
        };

        println!("connection to {} success", addr);

        result
    }

    pub fn write(&mut self, data: &[u8]) {
        match self {
            Self::Udp(socket) => {
                socket.send(data).unwrap();
            }
            Self::Tcp(socket) => socket.write_all(data).unwrap(),
        };
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Option<usize> {
        let readed = match self {
            Self::Udp(socket) => match socket.recv(buf) {
                Ok(readed) => readed,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(e) => panic!("{e:?}"),
            },
            Self::Tcp(socket) => match socket.read(buf) {
                Ok(readed) => readed,
                Err(e) if e.kind() == ErrorKind::WouldBlock => return None,
                Err(e) => panic!("{e:?}"),
            },
        };

        Some(readed)
    }
}
