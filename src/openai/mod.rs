use alloc::{borrow::ToOwned, format, string::String};
use drogue_network::addr::HostSocketAddr;
use embedded_tls::blocking::{Aes128GcmSha256, NoVerify, TlsConfig, TlsConnection, TlsContext};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use regex::Regex;

use crate::{
    net::{dns::DnsResolver, socket::tcp::TcpSocket},
    openai::types::CompletionResponse,
};
use constants::*;

use self::types::ChatHistory;

pub mod constants;
pub mod types;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAiError {
    CannotOpenSocket,
    CannotResolveHost,
    CannotConnect,
    TlsError,
    UnparsableResponseCode,
    UnparsableResponseBody,
    ResponseCodeNotOk,
}

pub struct OpenAiContext<'a> {
    tls_connection: TlsConnection<'a, TcpSocket, Aes128GcmSha256>,
}

impl<'a> OpenAiContext<'a> {
    /// Create a new OpenAI context
    ///
    /// # Example
    /// ```no_run
    /// let buf = &mut OpenAiContext::create_new_buf();
    /// let openai_context = OpenAiContext::new(&mut resolver, buf).unwrap();
    /// ```
    pub fn new<'b>(
        resolver: &'a mut DnsResolver,
        rng: &'b mut ChaCha20Rng,
        record_read_buf: &'a mut [u8],
        record_write_buf: &'a mut [u8],
    ) -> Result<Self, OpenAiError>
    where
        'b: 'a,
    {
        let socket = TcpSocket::open().map_err(|_| OpenAiError::CannotOpenSocket)?;
        Self::connect(&socket, resolver)?;

        let tls_config = TlsConfig::new().with_server_name(OPENAI_API_HOST);
        let tls_context = TlsContext::new(&tls_config, rng);

        let mut tls_connection = TlsConnection::new(socket, record_read_buf, record_write_buf);

        tls_connection
            .open::<ChaCha20Rng, NoVerify>(tls_context)
            .map_err(|_| OpenAiError::TlsError)?;

        Ok(OpenAiContext { tls_connection })
    }

    fn connect(socket: &TcpSocket, resolver: &mut DnsResolver) -> Result<(), OpenAiError> {
        let addr = resolver
            .resolve_with_google_dns(OPENAI_API_HOST)
            .map_err(|_| OpenAiError::CannotResolveHost)?;

        psp::dprintln!("resolved to {}", addr.0);

        let remote = HostSocketAddr::from(&DnsResolver::in_addr_to_string(addr), OPENAI_API_PORT)
            .map_err(|_| OpenAiError::CannotResolveHost)?;

        psp::dprintln!("connecting to {}", remote.addr().ip());

        socket
            .connect(remote)
            .map_err(|_| OpenAiError::CannotConnect)?;

        psp::dprintln!("connected");

        Ok(())
    }

    pub fn create_new_buf() -> [u8; 16_384] {
        [0; 16_384]
    }

    pub fn create_rng() -> ChaCha20Rng {
        let mut seed = 0;
        unsafe {
            psp::sys::sceRtcGetCurrentTick(&mut seed);
        }
        ChaCha20Rng::seed_from_u64(seed)
    }
}

pub struct OpenAi<'a> {
    api_key: String,
    openai_context: OpenAiContext<'a>,
    history: ChatHistory,
}

impl<'a> OpenAi<'a> {
    pub fn new(api_key: &str, openai_context: OpenAiContext<'a>) -> Result<Self, OpenAiError> {
        Ok(OpenAi {
            api_key: api_key.to_owned(),
            openai_context,
            history: ChatHistory::new_gpt3(0.7),
        })
    }

    pub fn ask_gpt(mut self, prompt: &str) -> Result<String, OpenAiError> {
        self.history.add_user_message(prompt.to_owned());

        let request_body = self.history.to_string();
        let content_length = request_body.len();

        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            PATH,
            OPENAI_API_HOST,
            self.api_key,
            content_length,
            request_body,
        );

        let request_bytes = request.as_bytes();

        self.openai_context
            .tls_connection
            .write(request_bytes)
            .map_err(|_| OpenAiError::TlsError)?;

        let response_buf = &mut [0u8; 16_384];
        self.openai_context
            .tls_connection
            .read(response_buf)
            .map_err(|_| OpenAiError::TlsError)?;

        let mut text = unsafe { alloc::string::String::from_utf8_unchecked(response_buf.to_vec()) };
        text = text.replace("\r", "");
        text = text.replace("\0", "");

        let res_code = text
            .split("\n")
            .next()
            .unwrap()
            .split(" ")
            .nth(1)
            .ok_or(OpenAiError::UnparsableResponseCode)?;
        if res_code != "200" {
            return Err(OpenAiError::ResponseCodeNotOk);
        }

        let res_body = Regex::new(r"\{.*\}")
            .unwrap()
            .find(&text)
            .ok_or(OpenAiError::UnparsableResponseBody)?
            .as_str();
        let completion_response: CompletionResponse = serde_json_core::from_str(res_body)
            .map_err(|_| OpenAiError::UnparsableResponseBody)?
            .0;

        let assistant_message = completion_response.choices[0].message.content.trim();
        self.history
            .add_assistant_message(assistant_message.to_owned());

        todo!()
    }
}
