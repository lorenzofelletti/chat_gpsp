use alloc::{format, string::String, vec as a_vec, vec::Vec};

use dns_protocol::{Flags, ResourceRecord};
use drogue_network::addr::HostSocketAddr;
use embedded_io::{Read, Write};
use psp::sys::in_addr;

use crate::net::socket::udp::UdpSocketState;

use super::{dns::google_dns_host_socket_addr, socket::udp::UdpSocket};

#[derive(Clone)]
pub struct DnsResolver {
    udp_socket: UdpSocket,
}

impl DnsResolver {
    /// Create a new DNS resolver
    #[allow(unused)]
    pub fn new(host: HostSocketAddr) -> Result<Self, ()> {
        let mut udp_socket = UdpSocket::open().map_err(|_| ())?;
        udp_socket.bind(Some(host)).map_err(|_| ())?;

        Ok(DnsResolver { udp_socket })
    }

    /// Create a new DNS resolver with default settings
    pub fn default() -> Result<Self, ()> {
        let mut udp_socket = UdpSocket::open().map_err(|_| ())?;
        udp_socket.bind(None).map_err(|_| ())?;

        Ok(DnsResolver { udp_socket })
    }

    /// Resolve a hostname to an IP address
    ///
    /// # Parameters
    /// - `host`: The hostname to resolve
    ///
    /// # Returns
    /// - `Ok(in_addr)`: The IP address of the hostname
    /// - `Err(())`: If the hostname could not be resolved
    pub fn resolve(&mut self, host: &str, dns_server: HostSocketAddr) -> Result<in_addr, ()> {
        // connect to the DNS server, if not already
        if self.udp_socket.get_socket_state() != UdpSocketState::Connected {
            self.udp_socket.connect(dns_server).map_err(|_| ())?;
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

        psp::dprintln!("writing request to buffer");

        // send the message to the DNS server
        let sent_bytes = self.udp_socket.write(&tx_buf).map_err(|_| ())?;

        psp::dprintln!(
            "wrote {} bytes, of {} bytes",
            sent_bytes,
            query.space_needed()
        );

        let mut rx_buf = [0u8; 1024];

        psp::dprintln!("reading response from buffer");

        // receive the response from the DNS server
        let data_len = self.udp_socket.read(&mut rx_buf).map_err(|_| ())?;

        psp::dprintln!("reading response from buffer");

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

        psp::dprintln!("message parsed");

        psp::dprintln!("answer: {:?}", message.answers());
        if message.answers().len() < 1 {
            return Err(());
        }
        let answer = message.answers()[0];

        match answer.data().len() {
            4 => {
                let mut octets = [0u8; 4];
                octets.copy_from_slice(answer.data());
                let addr = in_addr(u32::from_le_bytes(octets));
                psp::dprintln!("addr: {}", addr.0);
                Ok(addr)
            }
            _ => Err(()),
        }
    }

    #[inline]
    /// Resolve a hostname to an IP address using Google's DNS server `8.8.8.8`
    pub fn resolve_with_google_dns(&mut self, host: &str) -> Result<in_addr, ()> {
        let dns_server = google_dns_host_socket_addr();
        self.resolve(host, dns_server)
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
    /// ```no_run
    /// use psp::sys::in_addr;
    /// use crate::net::resolver::DnsResolver;
    ///
    /// let mut resolver = DnsResolver::default().unwrap();
    /// let addr = resolver.resolve("google.com").unwrap();
    /// let addr_str = resolver.in_addr_to_string(addr);
    /// ```
    ///
    pub fn in_addr_to_string(addr: in_addr) -> String {
        let octets = addr.0.to_le_bytes();
        let octets = octets
            .iter()
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>();
        octets.join(".")
    }
}
