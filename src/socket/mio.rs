use std::io;

#[cfg(unix)]
use std::os::unix::io::{AsRawFd, RawFd};

use mio::{Evented, Poll, PollOpt, Ready, Token};
#[cfg(unix)]
use mio::unix::EventedFd;
use socket2::{Domain, Protocol, SockAddr, Socket as Socket2, Type};
#[cfg(not(unix))]
use std::net::SocketAddr;

#[cfg(not(unix))]
use mio::net::UdpSocket;

pub struct Socket {
    #[cfg(unix)]
    socket: Socket2,
    #[cfg(not(unix))]
    socket: UdpSocket,
}


impl Socket {
    #[cfg(unix)]
    pub fn new(domain: Domain, type_: Type, protocol: Protocol) -> io::Result<Self> {
        let socket = Socket2::new(domain, type_, Some(protocol))?;
        socket.set_nonblocking(true)?;

        Ok(Self { socket: socket })
    }

    #[cfg(not(unix))]
    pub fn new(domain: Domain, type_: Type, protocol: Protocol) -> io::Result<Self> {
        let socket = Socket2::new(domain, type_, Some(protocol))?;
        socket.set_nonblocking(true)?;
        
        // panic ?
        let new_sock = mio::net::UdpSocket::from_socket(socket.into_udp_socket()).unwrap();
        Ok(Self { socket: new_sock })
    }

    #[cfg(not(unix))]
    pub fn as_udpsocket(&self) -> &UdpSocket {
        &self.socket
    }

    #[cfg(unix)]
    pub fn send_to(&self, buf: &[u8], target: &SockAddr) -> io::Result<usize> {
        self.socket.send_to(buf, target)
    }

    #[cfg(not(unix))]
    pub fn send_to(&self, buf: &[u8], target: &SockAddr) -> io::Result<usize> {
      let tgt_addr; 
        if let Some(addr) = target.as_inet() {
            tgt_addr = SocketAddr::V4(addr);
        } else if let Some(addr) = target.as_inet6() {
            tgt_addr = SocketAddr::V6(addr);
        } else {
            return Err( std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, 
                    format!("Invalid Socket ({:?})", target)));
        };
        self.socket.send_to(buf, &tgt_addr)
    }


    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.socket.recv(buf)
    }
}

#[cfg(unix)]
impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket.as_raw_fd()
    }
}

#[cfg(unix)]
impl Evented for Socket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

#[cfg(not(unix))]
impl Evented for Socket {
    fn register(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
//        self.socket.registration.register(poll, token, interest, opts)
//        self.socket.register(poll, token, interest, opts)
        poll.register(&self.socket, token, interest, opts)
    }

    fn reregister(
        &self,
        poll: &Poll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        poll.reregister(&self.socket, token, interest, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        poll.deregister(&self.socket)
    }
}