use alloc::{boxed::Box, vec::Vec};
use embedded_nal::{IpAddr, Ipv4Addr, SocketAddr};
use psp::sys::{self, sockaddr, socklen_t};

use core::ffi::c_void;

use crate::net::traits::SocketBuffer;

use super::{super::netc, error::SocketError, ToSockaddr};

#[derive(Clone, Copy, PartialEq, Eq)]
/// The state of a [`UdpSocket`]
pub enum UdpSocketState {
    /// The socket is not yet bound (the bind method has not been called)
    Unbound,
    /// The socket is bound (the bind method has been called)
    Bound,
    /// The socket is connected
    Connected,
}

#[repr(C)]
/// A UDP socket
///
/// # Fields
/// - [`UdpSocket::0`]: The socket file descriptor
/// - [`UdpSocket::1`]: The remote host to connect to
/// - [`UdpSocket::2`]: The state of the socket
/// - [`UdpSocket::3`]: The buffer to store data to send
///
/// # Notes
/// - Remote [host](Self::1) is set when the socket is bound calling [`bind()`](UdpSocket::bind)
pub struct UdpSocket(i32, Option<sockaddr>, UdpSocketState, Box<dyn SocketBuffer>);

impl UdpSocket {
    #[allow(dead_code)]
    /// Open a socket
    pub fn open() -> Result<UdpSocket, SocketError> {
        let sock = unsafe { sys::sceNetInetSocket(netc::AF_INET as i32, netc::SOCK_DGRAM, 0) };
        if sock < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            Ok(UdpSocket(
                sock,
                None,
                UdpSocketState::Unbound,
                Box::new(Vec::new()),
            ))
        }
    }

    #[allow(unused)]
    /// Bind the socket
    ///
    /// # Parameters
    /// - `addr`: The address to bind to, if `None` bind to `0.0.0.0:0`
    ///
    /// # Returns
    /// - `Ok(())` if the binding was successful
    /// - `Err(String)` if the binding was unsuccessful.
    pub fn bind(&mut self, addr: Option<SocketAddr>) -> Result<(), SocketError> {
        if self.2 != UdpSocketState::Unbound {
            return Err(SocketError::AlreadyBound);
        }

        let default_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 01)), 0);
        let addr = addr.unwrap_or(default_addr);
        match addr {
            SocketAddr::V4(v4) => {
                let sockaddr = v4.to_sockaddr();

                if unsafe {
                    sys::sceNetInetBind(
                        self.0,
                        &sockaddr,
                        core::mem::size_of::<netc::sockaddr>() as u32,
                    )
                } != 0
                {
                    let errno = unsafe { sys::sceNetInetGetErrno() };
                    Err(SocketError::Errno(errno))
                } else {
                    self.1 = Some(sockaddr);
                    self.2 = UdpSocketState::Bound;
                    Ok(())
                }
            }
            SocketAddr::V6(_) => Err(SocketError::UnsupportedAddressFamily),
        }
    }

    #[allow(unused)]
    /// Connect to a remote host
    ///
    /// # Notes
    /// The socket must be in state [`UdpSocketState::Bound`] to connect to a remote host.
    pub fn connect(&mut self, addr: SocketAddr) -> Result<(), SocketError> {
        match self.2 {
            UdpSocketState::Unbound => return Err(SocketError::NotBound),
            UdpSocketState::Bound => {}
            UdpSocketState::Connected => return Err(SocketError::AlreadyConnected),
        }

        match addr {
            SocketAddr::V4(v4) => {
                let sockaddr = super::socket_addr_v4_to_sockaddr(v4);

                if unsafe { sys::sceNetInetConnect(self.0, &sockaddr, Self::socket_len()) } != 0 {
                    let errno = unsafe { sys::sceNetInetGetErrno() };
                    Err(SocketError::Errno(errno))
                } else {
                    self.1 = Some(sockaddr);
                    self.2 = UdpSocketState::Connected;
                    Ok(())
                }
            }
            SocketAddr::V6(_) => Err(SocketError::UnsupportedAddressFamily),
        }
    }

    #[allow(unused)]
    /// Read from a socket in state [`UdpSocketState::Connected`]
    fn _read(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        if self.2 != UdpSocketState::Connected {
            return Err(SocketError::NotConnected);
        }
        let mut sockaddr = self.1.ok_or(SocketError::Other)?;
        let result =
            unsafe { sys::sceNetInetRecv(self.0, buf.as_mut_ptr() as *mut c_void, buf.len(), 0) };
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            Ok(result as usize)
        }
    }

    #[allow(unused)]
    /// Write to a socket in state [`UdpSocketState::Bound`]
    fn _read_from(&mut self, buf: &mut [u8]) -> Result<usize, SocketError> {
        match self.2 {
            UdpSocketState::Unbound => return Err(SocketError::NotBound),
            UdpSocketState::Bound => {}
            UdpSocketState::Connected => return Err(SocketError::AlreadyConnected),
        }
        let mut sockaddr = self.1.ok_or(SocketError::Other)?;
        let result = unsafe {
            sys::sceNetInetRecvfrom(
                self.0,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                0,
                &mut sockaddr,
                &mut Self::socket_len(),
            )
        };
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            Ok(result as usize)
        }
    }

    #[allow(unused)]
    /// Write to a socket in state [`UdpSocketState::Bound`]
    fn _write_to(&mut self, buf: &[u8], len: usize, to: SocketAddr) -> Result<usize, SocketError> {
        match self.2 {
            UdpSocketState::Unbound => return Err(SocketError::NotBound),
            UdpSocketState::Bound => {}
            UdpSocketState::Connected => return Err(SocketError::AlreadyConnected),
        }

        let sockaddr = match to {
            SocketAddr::V4(v4) => Ok(super::socket_addr_v4_to_sockaddr(v4)),
            SocketAddr::V6(_) => Err(SocketError::UnsupportedAddressFamily),
        }?;
        let socklen = core::mem::size_of::<netc::sockaddr>() as u32;

        self.3.append_buffer(buf);

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
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            self.3.shift_left_buffer(result as usize);
            Ok(result as usize)
        }
    }

    #[allow(unused)]
    /// Write to a socket in state [`UdpSocketState::Connected`]
    fn _write(&mut self, buf: &[u8]) -> Result<usize, SocketError> {
        if self.2 != UdpSocketState::Connected {
            return Err(SocketError::NotConnected);
        }

        self.3.append_buffer(buf);

        self.send()
    }

    fn _flush(&mut self) -> Result<(), SocketError> {
        if self.2 != UdpSocketState::Connected {
            return Err(SocketError::NotConnected);
        }

        while !self.3.is_empty() {
            self.send()?;
        }
        Ok(())
    }

    fn send(&mut self) -> Result<usize, SocketError> {
        let result = unsafe {
            sys::sceNetInetSend(
                self.0,
                self.3.as_slice().as_ptr() as *const c_void,
                self.3.len(),
                0,
            )
        };
        if (result as i32) < 0 {
            Err(SocketError::Errno(unsafe { sys::sceNetInetGetErrno() }))
        } else {
            self.3.shift_left_buffer(result as usize);
            Ok(result as usize)
        }
    }

    /// Get the state of the socket
    ///
    /// # Returns
    /// The state of the socket
    pub fn get_socket_state(&self) -> UdpSocketState {
        self.2
    }

    fn socket_len() -> socklen_t {
        core::mem::size_of::<netc::sockaddr>() as u32
    }
}

impl embedded_io::ErrorType for UdpSocket {
    type Error = SocketError;
}

impl embedded_io::Read for UdpSocket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        match self.get_socket_state() {
            UdpSocketState::Unbound => Err(SocketError::NotBound),
            UdpSocketState::Bound => self._read_from(buf),
            UdpSocketState::Connected => self._read(buf),
        }
    }
}

impl embedded_io::Write for UdpSocket {
    /// Write to the socket
    ///
    /// # Notes
    /// If the socket is not in state [`UdpSocketState::Connected`] this will return an error.
    /// To connect to a remote host use [`connect`](UdpSocket::connect) first.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        match self.get_socket_state() {
            UdpSocketState::Unbound => Err(SocketError::NotBound),
            UdpSocketState::Bound => Err(SocketError::NotConnected),
            UdpSocketState::Connected => self._write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        match self.get_socket_state() {
            UdpSocketState::Unbound => Err(SocketError::NotBound),
            UdpSocketState::Bound => Err(SocketError::NotConnected),
            UdpSocketState::Connected => self._flush(),
        }
    }
}
