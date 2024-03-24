use core::fmt::Debug;

use alloc::string::String;

use embedded_nal::SocketAddr;
use psp::sys::in_addr;

// trait defining a type that can resolve a hostname to an IP address
pub trait ResolveHostname {
    type Error: Debug;
    fn resolve_hostname(&mut self, hostname: &str) -> Result<SocketAddr, Self::Error>;
}

pub trait ResolveAddr {
    type Error: Debug;
    fn resolve_addr(&mut self, addr: in_addr) -> Result<String, Self::Error>;
}

pub trait DnsResolver: ResolveHostname + ResolveAddr {}
