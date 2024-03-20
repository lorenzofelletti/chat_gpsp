use alloc::boxed::Box;
use alloc::vec::Vec;
use embedded_io::ErrorType;

use embedded_nal::SocketAddr;
use psp::sys;

use core::ffi::c_void;

use crate::net::traits::SocketBuffer;

use super::super::netc;

use super::error::SocketError;

// TODO: review implementation
#[repr(C)]
/// A TCP socket
///
/// # Fields
/// - [`Self::0`]: The socket file descriptor
/// - [`Self::1`]: Whether the socket is connected
/// - [`Self::2`]: The buffer to store data to send
///
/// # Safety
/// This is a wrapper around a raw socket file descriptor.
///
/// The socket is closed when the struct is dropped.
pub struct TcpSocket(i32, bool, Box<dyn SocketBuffer>);

impl TcpSocket {
    #[allow(dead_code)]
    /// Open a TCP socket
    pub fn open() -> Result<TcpSocket, SocketError> {
        let sock = unsafe { sys::sceNetInetSocket(netc::AF_INET as i32, netc::SOCK_STREAM, 0) };
        if sock < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            Ok(TcpSocket(sock, false, Box::new(Vec::new())))
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
    pub fn connect(&mut self, remote: SocketAddr) -> Result<(), SocketError> {
        if self.1 {
            return Err(SocketError::AlreadyConnected);
        }
        match remote {
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
                    Err(SocketError::Errno(errno))
                } else {
                    self.1 = true;
                    Ok(())
                }
            }
            SocketAddr::V6(_) => Err(SocketError::UnsupportedAddressFamily),
        }
    }

    pub fn get_socket(&self) -> i32 {
        self.0
    }

    /// Read from the socket
    fn _read(&self, buf: &mut [u8]) -> Result<usize, SocketError> {
        let result =
            unsafe { sys::sceNetInetRecv(self.0, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) };
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            Ok(result)
        }
    }

    /// Write to the socket
    fn _write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        self.2.append_buffer(buf);
        self.send()
    }

    fn _flush(&mut self) -> Result<(), SocketError> {
        while !self.2.is_empty() {
            self.send()?;
        }
        Ok(())
    }

    fn send(&mut self) -> Result<usize, SocketError> {
        if !self.1 {
            return Err(SocketError::NotConnected);
        }

        let result = unsafe {
            sys::sceNetInetSend(
                self.0,
                self.2.as_slice().as_ptr() as *const c_void,
                self.2.len(),
                0,
            )
        };
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            self.2.shift_left_buffer(result as usize);
            Ok(result as usize)
        }
    }
}

impl ErrorType for TcpSocket {
    type Error = SocketError;
}

impl embedded_io::Read for TcpSocket {
    /// Read from the socket
    fn read<'m>(&'m mut self, buf: &'m mut [u8]) -> Result<usize, Self::Error> {
        if !self.1 {
            return Err(SocketError::NotConnected);
        }
        self._read(buf)
    }
}

impl embedded_io::Write for TcpSocket {
    /// Write to the socket
    fn write<'m>(&'m mut self, buf: &'m [u8]) -> Result<usize, Self::Error> {
        if !self.1 {
            return Err(SocketError::NotConnected);
        }
        self._write(buf)
    }

    fn flush(&mut self) -> Result<(), SocketError> {
        self._flush()
    }
}
