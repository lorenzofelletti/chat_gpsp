use core::ffi::c_void;

use alloc::{format, string::String, vec::Vec};
use psp::sys::{in_addr, sceNetResolverCreate, sceNetResolverDelete, sceNetResolverStartNtoA};

#[derive(Debug, Clone)]
pub struct DnsResolver {
    rid: i32,
}

impl DnsResolver {
    /// Create a new DNS resolver
    pub fn new() -> Result<Self, ()> {
        let rid = -1;
        let mut buf: Vec<u8> = Vec::with_capacity(1024);
        let buf_ptr = buf.as_mut_ptr() as *mut c_void;
        psp::dprintln!("Creating resolver...");
        unsafe {
            if sceNetResolverCreate(rid as *mut i32, buf_ptr, buf.capacity() as u32) < 0 {
                psp::dprintln!("sceNetResolverCreate failed");
                return Err(());
            }
        }
        psp::dprintln!("here");
        Ok(DnsResolver { rid })
    }

    /// Resolve a hostname to an IP address
    ///
    /// # Parameters
    /// - `host`: The hostname to resolve
    ///
    /// # Returns
    /// - `Ok(in_addr)`: The IP address of the hostname
    /// - `Err(())`: If the hostname could not be resolved
    pub fn resolve(&mut self, host: &str) -> Result<in_addr, ()> {
        let host_ptr = host.as_ptr();
        let addr = &mut in_addr(0);
        unsafe {
            if sceNetResolverStartNtoA(self.rid, host_ptr, addr as *mut in_addr, 2, 3) < 0 {
                return Err(());
            }
        }

        Ok(in_addr(addr.0))
    }

    /// Convert an [`in_addr`] to a [String]
    ///
    /// # Parameters
    /// - `addr`: The [`in_addr`] to convert
    ///
    /// # Returns
    /// - A [String] representation of the [`in_addr`]
    ///
    /// # Example
    /// ```
    /// use psp::sys::in_addr;
    /// use crate::net::resolver::DnsResolver;
    ///
    /// let mut resolver = DnsResolver::new().unwrap();
    /// let addr = resolver.resolve("google.com").unwrap();
    /// let addr_str = resolver.in_addr_to_string(addr);
    /// ```
    ///
    pub fn in_addr_to_string(&self, addr: in_addr) -> String {
        let octets = addr.0.to_le_bytes();
        let octets = octets
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>();
        octets.join(".")
    }
}

impl Drop for DnsResolver {
    fn drop(&mut self) {
        unsafe {
            sceNetResolverDelete(self.rid);
        }
    }
}
