use dns_protocol::Question;
use drogue_network::addr::{HostAddr, HostSocketAddr};

#[allow(unused)]
pub const GOOGLE_DNS_HOST_ADDR: [u8; 4] = [8, 8, 8, 8];
#[allow(unused)]
pub const GOOGLE_DNS_HOST_PORT: u16 = 53;

#[allow(unused)]
pub fn google_dns_host_socket_addr() -> HostSocketAddr {
    HostSocketAddr::new(HostAddr::ipv4(GOOGLE_DNS_HOST_ADDR), GOOGLE_DNS_HOST_PORT)
}

#[allow(unused)]
/// Create a DNS query for an A record
pub fn create_a_type_query(domain: &str) -> Question {
    Question::new(domain, dns_protocol::ResourceType::A, 1)
}
