pub const OPENAI_API_HOST: &str = "api.openai.com";
pub const POST_PATH: &str = "/v1/chat/completions";
pub const GPT3_MODEL: &str = "gpt-3.5-turbo";
#[allow(unused)]
pub const CHAT_MAX_LENGTH: u16 = 128;
#[allow(unused)]
pub const CHAT_MAX_LENGTH_USIZE: usize = CHAT_MAX_LENGTH as usize;
#[allow(unused)]
pub const MAX_MESSAGES_IN_A_REQUEST: usize = 10;

pub const MAX_RETRIES: usize = 1;
