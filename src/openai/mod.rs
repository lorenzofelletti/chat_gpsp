use alloc::{borrow::ToOwned, format, string::{String, ToString}};
use drogue_network::addr::HostSocketAddr;

use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use regex::Regex;

use crate::{
    net::{dns::DnsResolver, socket::{tcp::TcpSocket, tls::TlsSocket}},
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
    tls_socket: TlsSocket<'a>,
}

impl<'a> OpenAiContext<'a> {
    /// Create a new OpenAI context
    ///
    /// # Example
    /// ```no_run
    /// let mut read_buf = OpenAiContext::create_new_buf();
    /// let mut write_buf = OpenAiContext::create_new_buf();
    /// let openai_context = OpenAiContext::new(&mut resolver, &mut read_buf, &mut write_buf).unwrap();
    /// ```
    pub fn new<'b>(
        resolver: &'a mut DnsResolver,
        record_read_buf: &'a mut [u8],
        record_write_buf: &'a mut [u8],
    ) -> Result<Self, OpenAiError>
    where
        'b: 'a,
    {
        let socket = TcpSocket::open().map_err(|_| OpenAiError::CannotOpenSocket)?;
        psp::dprintln!("opened tcp socket");
        Self::connect(&socket, resolver)?;
        psp::dprintln!("connected to openai");

        let mut tls_socket = TlsSocket::new(socket, record_read_buf, record_write_buf, OPENAI_API_HOST);

        tls_socket.open(Self::generate_seed()).map_err(|_| OpenAiError::TlsError)?;

        psp::dprintln!("opened tls connection");

        Ok(OpenAiContext { tls_socket})
    }

    fn connect(socket: &TcpSocket, resolver: &mut DnsResolver) -> Result<(), OpenAiError> {
        let addr = resolver
            .resolve_with_google_dns(OPENAI_API_HOST)
            .map_err(|_| OpenAiError::CannotResolveHost)?;

        psp::dprintln!("resolved to {}", addr.0);

        let remote = HostSocketAddr::from(&DnsResolver::in_addr_to_string(addr), HTTPS_PORT)
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

    pub fn generate_seed() -> u64 {
        let mut seed = 0;
        unsafe {
            psp::sys::sceRtcGetCurrentTick(&mut seed);
        }
        seed
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

    pub fn ask_gpt(&mut self, prompt: &str) -> Result<String, OpenAiError> {
        self.history.add_user_message(prompt.to_owned());

        let (request_body, content_length) = self.history.to_string_with_content_length();

        let request = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nUser-Agent: Sony PSP\r\n\r\n{}\r\n",
            POST_PATH,  
            OPENAI_API_HOST,
            self.api_key,
            content_length,
            request_body,
        );

        let request_bytes = request.as_bytes();

        self.openai_context
            .tls_socket
            .write_all(request_bytes)
            .map_err(|_| OpenAiError::TlsError)?;
        self.openai_context.tls_socket.flush().map_err(|_| OpenAiError::TlsError)?;

        let response_buf = &mut [0u8; 16_384];
        self.openai_context
            .tls_socket
            .read(response_buf)
            .map_err(|_| OpenAiError::TlsError)?;

        let mut text = String::from_utf8_lossy(response_buf).to_string();
        text = text.replace("\r", "");
        text = text.replace("\0", "");

        psp::dprintln!("{}", text);

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
