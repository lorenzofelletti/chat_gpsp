#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketError {
    /// The socket is not connected
    NotConnected,
    /// The socket is already connected
    AlreadyConnected,
    /// The socket is already bound
    AlreadyBound,
    /// The socket is not bound
    NotBound,
    /// Unsupported address family
    UnsupportedAddressFamily,
    /// Socket error with errno
    Errno(i32),
    /// Other error
    Other,
}

impl embedded_io::Error for SocketError {
    fn kind(&self) -> embedded_io::ErrorKind {
        match self {
            SocketError::NotConnected => embedded_io::ErrorKind::NotConnected,
            SocketError::UnsupportedAddressFamily => embedded_io::ErrorKind::Unsupported,
            _ => embedded_io::ErrorKind::Other,
        }
    }
}
