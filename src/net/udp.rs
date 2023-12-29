use alloc::{format, string::String};
use drogue_network::addr::{HostAddr, HostSocketAddr, SocketAddr, SocketAddrV4};
use psp::sys::{self, sockaddr, socklen_t};

use core::ffi::c_void;

use super::netc;

#[derive(Clone, Copy)]
#[repr(C)]
/// A UDP socket
///
/// # Fields
/// - `0`: The socket file descriptor
/// - `1`: The remote host to connect to
/// - `2`: The length of the remote host
///
/// # Notes
/// - The remote host and length are set when the socket is bound calling [`self::bind()`]
pub struct UdpSocket(i32, Option<sockaddr>, Option<socklen_t>);

impl UdpSocket {
    #[allow(dead_code)]
    /// Open a socket
    pub fn open() -> Result<UdpSocket, ()> {
        let sock = unsafe { sys::sceNetInetSocket(netc::AF_INET as i32, netc::SOCK_DGRAM, 0) };
        if sock < 0 {
            Err(())
        } else {
            Ok(UdpSocket(sock, None, None))
        }
    }

    #[allow(dead_code)]
    /// Bind the socket
    ///
    /// # Parameters
    /// - `addr`: The address to bind to, if `None` bind to `localhost:1234`
    ///
    /// # Returns
    /// - `Ok(())` if the binding was successful
    /// - `Err(String)` if the binding was unsuccessful.
    pub fn bind(&mut self, addr: Option<HostSocketAddr>) -> Result<(), String> {
        let localhost_addr = HostSocketAddr::new(HostAddr::ipv4([0, 0, 0, 0]), 0);
        let addr = addr.unwrap_or(localhost_addr);
        match addr.as_socket_addr() {
            SocketAddr::V4(v4) => {
                let sockaddr = Self::socket_addr_v4_to_sockaddr(v4);

                if unsafe {
                    sys::sceNetInetBind(
                        self.0,
                        &sockaddr,
                        core::mem::size_of::<netc::sockaddr>() as u32,
                    )
                } != 0
                {
                    let errno = unsafe { sys::sceNetInetGetErrno() };
                    Err(format!("0x{:08x}", errno))
                } else {
                    self.1 = Some(sockaddr);
                    self.2 = Some(core::mem::size_of::<netc::sockaddr>() as u32);
                    Ok(())
                }
            }
            SocketAddr::V6(_) => Err("IPv6 not supported".into()),
        }
    }

    #[allow(unused)]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut sockaddr = self.1.ok_or(())?;
        let mut socklen = self.2.ok_or(())?;
        psp::dprintln!("read() - b4 recvfrom - arrived here");
        let result =
            unsafe { sys::sceNetInetRecv(self.0, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) };
        psp::dprintln!("received UDP packet");
        if (result as i32) < 0 {
            psp::dprintln!("error: {}", unsafe { sys::sceNetInetGetErrno() });
            Err(())
        } else {
            Ok(result as usize)
        }
    }

    #[allow(unused)]
    pub fn read_from(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let mut sockaddr = self.1.ok_or(())?;
        let mut socklen = self.2.ok_or(())?;
        psp::dprintln!("read() - b4 recvfrom - arrived here");
        let result = unsafe {
            sys::sceNetInetRecvfrom(
                self.0,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                0,
                &mut sockaddr,
                &mut socklen,
            )
        };
        psp::dprintln!("received UDP packet");
        if (result as i32) < 0 {
            psp::dprintln!("error: {}", unsafe { sys::sceNetInetGetErrno() });
            Err(())
        } else {
            Ok(result as usize)
        }
    }

    #[allow(unused)]
    pub fn write_to(&self, buf: &[u8], len: usize, to: HostSocketAddr) -> Result<usize, ()> {
        let to: SocketAddr = to.as_socket_addr();
        let sockaddr = match to {
            SocketAddr::V4(v4) => Ok(Self::socket_addr_v4_to_sockaddr(v4)),
            SocketAddr::V6(_) => Err(()),
        }?;
        psp::dprintln!("arrived here");
        let socklen = core::mem::size_of::<netc::sockaddr>() as u32;
        let result = unsafe {
            sys::sceNetInetSendto(
                self.0,
                buf.as_ptr() as *const c_void,
                len,
                0,
                &sockaddr,
                socklen,
            )
        };
        psp::dprintln!("sent UDP packet");
        if (result as i32) < 0 {
            psp::dprintln!("error: {}", unsafe { sys::sceNetInetGetErrno() });
            Err(())
        } else {
            Ok(result as usize)
        }
    }

    #[allow(unused)]
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
}
