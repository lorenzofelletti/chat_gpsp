use alloc::{
    borrow::ToOwned,
    format,
    string::{String, ToString},
    vec::Vec,
};

use lazy_static::lazy_static;
use psp_net::{
    constants::HTTPS_PORT,
    http::types::{Authorization, ContentType},
    parse_response, request,
    traits::dns::ResolveHostname,
    types::SocketRecvFlags,
};
use psp_net::{
    socket::{error::*, SocketAddr},
    tls_socket,
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
    // CannotOpenSocket,
    CannotResolveHost,
    // CannotConnect,
    TlsError(String),
    UnparsableResponseCode(String),
    UnparsableResponseBody(String),
    PartialResponse(String),
    ResponseCodeNotOk,
}

impl OpenAiError {
    pub fn new_unparsable_response_code(msg: String) -> Self {
        OpenAiError::UnparsableResponseCode(msg)
    }
    pub fn new_empty_unparsable_response_code() -> Self {
        OpenAiError::UnparsableResponseCode(String::new())
    }
    pub fn new_empty_partial_response() -> Self {
        OpenAiError::PartialResponse(String::new())
    }
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

pub struct OpenAi {
    remote: SocketAddr,
    api_key: String,
    history: ChatHistory,
}

impl OpenAi {
    pub fn new(openai_context: &OpenAiContext) -> Result<Self, OpenAiError> {
        Ok(OpenAi {
            remote: openai_context.remote(),
            api_key: openai_context.api_key(),
            history: ChatHistory::new_gpt3(0.7),
        })
    }

    pub fn ask_gpt(&mut self, prompt: &str) -> Result<String, OpenAiError> {
        fn log_error(e: &TlsError) -> OpenAiError {
            OpenAiError::TlsError(format!("{:?}", e))
        }

        self.history.add_user_message(prompt.to_owned());

        tls_socket! {
            name: _try_socket,
            host OPENAI_API_HOST => &self.ip(),
            recv_flags SocketRecvFlags::MSG_PEEK,
        };

        let mut socket = match _try_socket {
            Ok(socket) => socket,
            Err(e) => return Err(OpenAiError::TlsError(format!("{:?}", e))),
        };

        let auth = Authorization::Bearer(self.api_key.clone());

        let request = request! {
            OPENAI_API_HOST post POST_PATH ContentType::ApplicationJson,
            authorization auth,
            body Vec::new(),
            "User-Agent" => "Sony PSP";
        }
        .render();

        psp_net::write!(request => socket).map_err(|e| log_error(&e))?;

        let response_string = psp_net::read!(string from socket).map_err(|e| log_error(&e))?;

        let res_code = parse_res_code(&response_string)?;
        if res_code != 200 {
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

    fn ip(&self) -> String {
        self.remote.ip().to_string()
    }
}

fn parse_res_code(response_string: &String) -> Result<usize, OpenAiError> {
    let parsed = parse_response!(response_string,);
    if parsed.is_err() {
        return Err(OpenAiError::UnparsableResponseCode(
            parsed.unwrap_err().to_string(),
        ));
    }
    let parsed = parsed.unwrap();
    if parsed.is_partial() {
        return Err(OpenAiError::new_empty_partial_response());
    }
    Ok(parsed.unwrap())
}
