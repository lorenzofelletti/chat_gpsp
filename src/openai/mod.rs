use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
};

use lazy_static::lazy_static;
use psp_net::socket::{error::*, state::Ready, tcp::TcpSocket, tls::TlsSocket, SocketAddr};
use psp_net::{
    constants::HTTPS_PORT,
    traits::{dns::ResolveHostname, io::Open, io::Read, io::Write},
    types::{SocketOptions, SocketRecvFlags, TlsSocketOptions},
};
use regex::{Regex, RegexBuilder};

use crate::openai::types::CompletionResponse;
use constants::*;

use self::types::ChatHistory;

pub mod constants;
pub mod types;

lazy_static! {
    static ref BODY_REGEX: Regex = RegexBuilder::new(r"\{.*\}")
        .dot_matches_new_line(true)
        .build()
        .expect("regex should be valid");
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenAiError {
    CannotOpenSocket,
    CannotResolveHost,
    CannotConnect,
    TlsError(String),
    UnparsableResponseCode,
    UnparsableResponseBody(String),
    ResponseCodeNotOk,
}

pub struct OpenAiContext {
    remote: SocketAddr,
    api_key: String,
}

impl OpenAiContext {
    /// Create a new OpenAI context
    ///
    /// # Example
    /// ```no_run
    /// let openai_context = OpenAiContext::new(&mut resolver, "my_api_key").unwrap();
    /// ```
    pub fn new<T>(resolver: &mut T, api_key: &str) -> Result<Self, OpenAiError>
    where
        T: ResolveHostname,
    {
        let api_key = api_key.to_owned();

        let mut remote = resolver
            .resolve_hostname(OPENAI_API_HOST)
            .map_err(|_| OpenAiError::CannotResolveHost)?;
        remote.set_port(HTTPS_PORT);

        Ok(OpenAiContext { remote, api_key })
    }

    pub fn api_key(&self) -> String {
        self.api_key.clone()
    }

    pub fn remote(&self) -> SocketAddr {
        self.remote
    }
}

pub struct OpenAi<'a> {
    remote: SocketAddr,
    api_key: String,
    history: ChatHistory,
    tls_socket_options: TlsSocketOptions<'a>,
}

impl<'a> OpenAi<'a> {
    pub fn new(openai_context: &OpenAiContext) -> Result<Self, OpenAiError> {
        Ok(OpenAi {
            remote: openai_context.remote(),
            api_key: openai_context.api_key(),
            history: ChatHistory::new_gpt3(0.7),
            tls_socket_options: Self::create_tls_socket_options(),
        })
    }

    pub fn ask_gpt(&mut self, prompt: &str) -> Result<String, OpenAiError> {
        fn log_error(e: &TlsError) -> OpenAiError {
            OpenAiError::TlsError(format!("{:?}", e))
        }

        self.history.add_user_message(prompt.to_owned());

        let (request_body, content_length) = self.history.to_string_with_content_length();

        let mut read_buf = Self::create_new_buf();
        let mut write_buf = Self::create_new_buf();

        // get tls socket
        let mut tls_socket = Self::open_tls_socket(
            &mut read_buf,
            &mut write_buf,
            self.remote,
            &self.tls_socket_options,
        )?;

        let request = format!(
            "POST {} HTTP/1.1\nHost: {}\nAuthorization: Bearer {}\nContent-Type: application/json\nContent-Length: {}\nUser-Agent: Sony PSP\n\n{}\n",
            POST_PATH,
            OPENAI_API_HOST,
            self.api_key,
            content_length,
            request_body,
        );
        let request_bytes = request.as_bytes();

        let mut response_string = String::new();

        tls_socket
            .write_all(request_bytes)
            .map_err(|e| log_error(&e))?;

        tls_socket.flush().map_err(|e| log_error(&e))?;

        for _ in 1..=MAX_RETRIES {
            let response_buf = &mut [0u8; 16_384];
            tls_socket.read(response_buf).map_err(|e| log_error(&e))?;

            let text = String::from_utf8_lossy(response_buf)
                .to_string()
                .replace(['\r', '\0'], "");

            if text.is_empty() {
                break;
            }

            response_string += &text;
        }

        let res_code = response_string
            .split('\n')
            .next()
            .ok_or(OpenAiError::UnparsableResponseCode)?
            .split(' ')
            .nth(1)
            .ok_or(OpenAiError::UnparsableResponseCode)?;
        if res_code != "200" {
            return Err(OpenAiError::ResponseCodeNotOk);
        }

        // find for double newline, get the body
        let mut res_body =
            response_string
                .split("\n\n")
                .nth(1)
                .ok_or(OpenAiError::UnparsableResponseBody(
                    "Body not found".to_owned(),
                ))?;

        // trim the body (lately some weird characters get added around)
        if let Some(res) = BODY_REGEX.find(res_body) {
            res_body = res.as_str();
        } else {
            return Err(OpenAiError::UnparsableResponseBody(
                "Malformed body".to_owned(),
            ));
        }

        let completion_response: CompletionResponse = serde_json_core::from_str(&res_body)
            .map_err(|e| OpenAiError::UnparsableResponseBody(e.to_string()))?
            .0;

        let assistant_message = completion_response.choices[0].message.content.trim();
        self.history
            .add_assistant_message(assistant_message.to_owned());

        Ok(assistant_message.to_owned())
    }

    fn open_tls_socket<'b>(
        record_read_buf: &'b mut [u8],
        record_write_buf: &'b mut [u8],
        remote: SocketAddr,
        tls_socket_options: &'b TlsSocketOptions<'b>,
    ) -> Result<TlsSocket<'b, Ready>, OpenAiError> {
        let socket = TcpSocket::new().map_err(|_| OpenAiError::CannotOpenSocket)?;
        let mut socket = socket
            .open(&SocketOptions::new(remote))
            .map_err(|_| OpenAiError::CannotConnect)?;
        // enable peeking (useful for TLS messages)
        socket.set_recv_flags(SocketRecvFlags::MSG_PEEK);

        let tls_socket = TlsSocket::new(socket, record_read_buf, record_write_buf);
        let tls_socket = tls_socket
            .open(&tls_socket_options)
            .map_err(|e| OpenAiError::TlsError(format!("{:?}", e)))?;
        Ok(tls_socket)
    }

    pub fn create_new_buf() -> [u8; 16_384] {
        [0; 16_384]
    }

    pub fn generate_seed() -> u64 {
        let mut seed = 0;
        unsafe {
            psp::sys::sceRtcGetCurrentTick(&mut seed);
        }
        seed
    }

    fn create_tls_socket_options() -> TlsSocketOptions<'static> {
        TlsSocketOptions::new(Self::generate_seed(), OPENAI_API_HOST.to_owned())
    }
}
