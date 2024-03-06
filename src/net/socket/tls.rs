use alloc::string::String;
use embedded_io::Write;
use embedded_tls::{
    blocking::TlsConnection, Aes128GcmSha256, Certificate, NoVerify, TlsConfig, TlsContext,
};
use psp::sys;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use regex::Regex;

use super::tcp::TcpSocket;

lazy_static::lazy_static! {
    static ref REGEX: Regex = Regex::new("\r|\0").unwrap();
}

pub struct TlsSocket<'a> {
    socket_fd: i32,
    tls_connection: TlsConnection<'a, TcpSocket, Aes128GcmSha256>,
    tls_config: TlsConfig<'a, Aes128GcmSha256>,
}

impl<'a> TlsSocket<'a> {
    pub fn new(
        socket: TcpSocket,
        record_read_buf: &'a mut [u8],
        record_write_buf: &'a mut [u8],
        server_name: &'a str,
        cert: Option<&'a [u8]>,
    ) -> Self {
        let tls_config: TlsConfig<'_, Aes128GcmSha256> = match cert {
            Some(cert) => TlsConfig::new()
                .with_server_name(server_name)
                .with_cert(Certificate::RawPublicKey(cert))
                .enable_rsa_signatures(),
            None => TlsConfig::new()
                .with_server_name(server_name)
                .enable_rsa_signatures(),
        };

        let tls_connection: TlsConnection<TcpSocket, Aes128GcmSha256> =
            TlsConnection::new(socket, record_read_buf, record_write_buf);

        TlsSocket {
            socket_fd: socket.get_socket(),
            tls_connection,
            tls_config,
        }
    }

    fn new_buffer() -> [u8; 16_384] {
        [0; 16_384]
    }

    pub fn open(&mut self, seed: u64) -> Result<(), embedded_tls::TlsError> {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let tls_context = TlsContext::new(&self.tls_config, &mut rng);
        self.tls_connection
            .open::<ChaCha20Rng, NoVerify>(tls_context)
    }

    #[allow(unused)]
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, embedded_tls::TlsError> {
        self.tls_connection.write(buf)
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), embedded_tls::TlsError> {
        self.tls_connection.write_all(buf)
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, embedded_tls::TlsError> {
        self.tls_connection.read(buf)
    }

    #[allow(unused)]
    pub fn read_string(&mut self) -> Result<String, embedded_tls::TlsError> {
        let mut buf = Self::new_buffer();
        let _ = self.read(&mut buf)?;

        let text = String::from_utf8_lossy(&buf);
        let text = REGEX.replace_all(&text, "");
        Ok(text.into_owned())
    }

    pub fn flush(&mut self) -> Result<(), embedded_tls::TlsError> {
        self.tls_connection.flush()
    }
}

impl Drop for TlsSocket<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::sceNetInetClose(self.socket_fd);
        }
    }
}
