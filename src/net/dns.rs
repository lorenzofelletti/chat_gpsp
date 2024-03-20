use alloc::{string::String, vec as a_vec};
use dns_protocol::{Flags, Question, ResourceRecord};
use embedded_io::{Read, Write};
use embedded_nal::{IpAddr, Ipv4Addr, SocketAddr};
use psp::sys::in_addr;

use crate::net::socket::udp::UdpSocketState;

use super::{socket::udp::UdpSocket, traits};

pub const DNS_PORT: u16 = 53;
lazy_static::lazy_static! {
    static ref GOOGLE_DNS_HOST: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), DNS_PORT);
}

#[allow(unused)]
/// Create a DNS query for an A record
pub fn create_a_type_query(domain: &str) -> Question {
    Question::new(domain, dns_protocol::ResourceType::A, 1)
}

/// A DNS resolver
pub struct DnsResolver {
    udp_socket: UdpSocket,
    dns: SocketAddr,
}

impl DnsResolver {
    /// Create a new DNS resolver
    #[allow(unused)]
    pub fn new(dns: SocketAddr) -> Result<Self, ()> {
        let mut udp_socket = UdpSocket::open().map_err(|_| ())?;
        udp_socket.bind(Some(dns)).map_err(|_| ())?;

        Ok(DnsResolver { udp_socket, dns })
    }

    /// Create a new DNS resolver with default settings.
    /// The default settings are to use Google's DNS server at `8.8.8.8:53`
    pub fn default() -> Result<Self, ()> {
        let dns = GOOGLE_DNS_HOST.clone();
        let mut udp_socket = UdpSocket::open().map_err(|_| ())?;
        udp_socket.bind(None).map_err(|_| ())?;

        Ok(DnsResolver { udp_socket, dns })
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
        // connect to the DNS server, if not already
        if self.udp_socket.get_socket_state() != UdpSocketState::Connected {
            self.udp_socket.connect(self.dns).map_err(|_| ())?;
        }

        // create a new query
        let mut questions = [super::dns::create_a_type_query(host)];
        let query = dns_protocol::Message::new(
            0x42,
            Flags::standard_query(),
            &mut questions,
            &mut [],
            &mut [],
            &mut [],
        );

        // create a new buffer with the size of the message
        let mut tx_buf = a_vec![0u8; query.space_needed()];
        // serialize the message into the buffer
        query.write(&mut tx_buf).map_err(|_| ())?;

        // send the message to the DNS server
        let _ = self.udp_socket.write(&tx_buf).map_err(|_| ())?;

        let mut rx_buf = [0u8; 1024];

        // receive the response from the DNS server
        let data_len = self.udp_socket.read(&mut rx_buf).map_err(|_| ())?;

        if data_len == 0 {
            return Err(());
        }

        // parse the response
        let mut answers = [ResourceRecord::default(); 16];
        let mut authority = [ResourceRecord::default(); 16];
        let mut additional = [ResourceRecord::default(); 16];
        let message = dns_protocol::Message::read(
            &rx_buf[..data_len],
            &mut questions,
            &mut answers,
            &mut authority,
            &mut additional,
        )
        .map_err(|_| ())?;

        if message.answers().is_empty() {
            return Err(());
        }
        let answer = message.answers()[0];

        match answer.data().len() {
            4 => {
                let mut octets = [0u8; 4];
                octets.copy_from_slice(answer.data());
                let addr = in_addr(u32::from_le_bytes(octets));
                Ok(addr)
            }
            _ => Err(()),
        }
    }
}

impl traits::dns::ResolveHostname for DnsResolver {
    type Error = ();

    fn resolve_hostname(&mut self, hostname: &str) -> Result<in_addr, ()> {
        self.resolve(hostname)
    }
}

impl traits::dns::ResolveAddr for DnsResolver {
    type Error = ();

    fn resolve_addr(&mut self, _addr: in_addr) -> Result<String, ()> {
        todo!("resolve_addr")
    }
}

impl traits::dns::DnsResolver for DnsResolver {}
