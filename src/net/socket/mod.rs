use embedded_nal::{Ipv4Addr, SocketAddr, SocketAddrV4};
use psp::sys::{in_addr, sockaddr};

use super::netc;

pub mod error;
pub mod tcp;
pub mod tls;
pub mod udp;

fn socket_addr_v4_to_sockaddr(addr: SocketAddrV4) -> sockaddr {
    let octets = addr.ip().octets();
    let sin_addr = u32::from_le_bytes(octets);
    let port = addr.port().to_be();

    let sockaddr_in = netc::sockaddr_in {
        sin_len: core::mem::size_of::<netc::sockaddr_in>() as u8,
        sin_family: netc::AF_INET,
        sin_port: port,
        sin_addr: netc::in_addr(sin_addr),
        sin_zero: [0u8; 8],
    };

    unsafe { core::mem::transmute::<netc::sockaddr_in, netc::sockaddr>(sockaddr_in) }
}

pub trait ToSockaddr {
    fn to_sockaddr(&self) -> sockaddr;
}

impl ToSockaddr for SocketAddrV4 {
    fn to_sockaddr(&self) -> sockaddr {
        socket_addr_v4_to_sockaddr(*self)
    }
}

pub trait ToSocketAddr {
    fn to_socket_addr(&self) -> SocketAddr;
}

impl ToSocketAddr for in_addr {
    fn to_socket_addr(&self) -> SocketAddr {
        let octets = self.0.to_be_bytes();
        let ip = Ipv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
        SocketAddr::V4(SocketAddrV4::new(ip, 0))
    }
}
