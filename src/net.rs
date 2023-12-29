use alloc::{format, string::String};
use drogue_network::addr::{HostSocketAddr, SocketAddr};
use embedded_io::ErrorType;
use embedded_tls::TlsError;
use psp::sys;

use core::ffi::c_void;

pub mod dns;
pub mod netc;
pub mod resolver;
pub mod udp;
pub mod utils;

#[derive(Clone, Copy)]
#[repr(C)]
/// A socket
pub struct Socket(i32);

impl Socket {
    #[allow(dead_code)]
    /// Open a socket
    pub fn open() -> Result<Socket, ()> {
        let sock = unsafe { sys::sceNetInetSocket(netc::AF_INET as i32, netc::SOCK_STREAM, 0) };
        if sock < 0 {
            Err(())
        } else {
            Ok(Socket(sock))
        }
    }

    #[allow(dead_code)]
    /// Connect to a remote host
    ///
    /// # Parameters
    /// - `remote`: The remote host to connect to
    ///
    /// # Returns
    /// - `Ok(())` if the connection was successful
    /// - `Err(String)` if the connection was unsuccessful.
    pub fn connect(&self, remote: HostSocketAddr) -> Result<(), String> {
        match remote.as_socket_addr() {
            SocketAddr::V4(v4) => {
                let octets = v4.ip().octets();
                let sin_addr = u32::from_le_bytes(octets);
                let port = v4.port().to_be();

                let sockaddr_in = netc::sockaddr_in {
                    sin_len: core::mem::size_of::<netc::sockaddr_in>() as u8,
                    sin_family: netc::AF_INET,
                    sin_port: port,
                    sin_addr: netc::in_addr(sin_addr),
                    sin_zero: [0u8; 8],
                };

                let sockaddr = unsafe {
                    core::mem::transmute::<netc::sockaddr_in, netc::sockaddr>(sockaddr_in)
                };

                if unsafe {
                    sys::sceNetInetConnect(
                        self.0,
                        &sockaddr,
                        core::mem::size_of::<netc::sockaddr_in>() as u32,
                    )
                } < 0
                {
                    let errno = unsafe { sys::sceNetInetGetErrno() };
                    Err(format!("0x{:08x}", errno))
                } else {
                    Ok(())
                }
            }
            SocketAddr::V6(_) => Err("IPv6 not supported".into()),
        }
    }

    fn _read(self, buf: &mut [u8]) -> Result<usize, ()> {
        let result =
            unsafe { sys::sceNetInetRecv(self.0, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) };
        if (result as i32) < 0 {
            Err(())
        } else {
            Ok(result)
        }
    }

    fn _write(&self, buf: &[u8]) -> Result<usize, ()> {
        let result =
            unsafe { sys::sceNetInetSend(self.0, buf.as_ptr() as *const c_void, buf.len(), 0) };
        if (result as i32) < 0 {
            Err(())
        } else {
            Ok(result as usize)
        }
    }
}

impl ErrorType for Socket {
    type Error = TlsError;
}

impl embedded_io::Read for Socket {
    /// Read from the socket
    fn read<'m>(&'m mut self, buf: &'m mut [u8]) -> Result<usize, TlsError> {
        self._read(buf).map_err(|_| TlsError::InternalError)
    }
}

impl embedded_io::Write for Socket {
    /// Write to the socket
    fn write<'m>(&'m mut self, buf: &'m [u8]) -> Result<usize, TlsError> {
        self._write(buf).map_err(|_| TlsError::InternalError)
    }

    fn flush(&mut self) -> Result<(), TlsError> {
        Ok(())
    }
}
