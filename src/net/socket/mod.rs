use drogue_network::addr::SocketAddrV4;
use psp::sys::sockaddr;

use super::netc;

pub mod error;
pub mod tcp;
pub mod udp;

#[allow(unused)]
pub fn socket_addr_v4_to_sockaddr(addr: SocketAddrV4) -> sockaddr {
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
