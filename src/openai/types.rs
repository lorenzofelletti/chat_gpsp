use crate::openai::constants::*;
use alloc::{borrow::ToOwned, format, string::String, vec::Vec};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    role: String,
    pub content: String,
}

impl Message {
    pub fn new_user(content: String) -> Self {
        Self {
            role: "user".to_owned(),
            content,
        }
    }
    pub fn new_assistant(content: String) -> Self {
        Self {
            role: "assistant".to_owned(),
            content,
        }
    }

    pub fn to_string(&self) -> String {
        let serialized_content = self
            .content
            .replace('\n', "\\n")
            .replace('\t', "\\t")
            .replace('\'', "\\'")
            .replace('\"', "\\\"");

        format!(
            "{{\"role\": \"{}\", \"content\": \"{}\"}}",
            self.role, serialized_content
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChatHistory {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

impl ChatHistory {
    pub fn new(model: String, temperature: f32) -> Self {
        Self {
            model,
            messages: Vec::new(),
            temperature,
        }
    }

    #[inline]
    pub fn new_gpt3(temperature: f32) -> Self {
        Self::new(GPT3_MODEL.to_owned(), temperature)
    }

    #[allow(unused)]
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(Message::new_user(content));
    }

    pub fn add_assistant_message(&mut self, content: String) {
        self.messages.push(Message::new_assistant(content));
    }

    pub fn to_string(&self) -> String {
        let mut messages = String::new();
        for message in &self.messages {
            messages.push_str(&message.to_string());
            messages.push(',');
        }
        messages.pop(); // remove last comma

        format!(
            "{{\n  \"model\": \"{}\",\n  \"messages\": [{}],\n  \"temperature\": {},\n  \"stream\": false\n}}",
            self.model, messages, self.temperature,
        )
    }

    pub fn to_string_with_content_length(&self) -> (String, usize) {
        let string = self.to_string();
        let len = string.len();

        (string, len)
    }
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

#[derive(Debug, Deserialize)]
pub struct ResponseMessage {
    pub role: heapless::String<32>,
    pub content: heapless::String<1024>,
}

#[derive(Debug, Deserialize)]
/// # Note
/// This struct does not support the [`Self::logprobs`] field different from `None` yet.
pub struct CompletionChoice {
    pub message: ResponseMessage,
    pub index: i64,
    pub finish_reason: heapless::String<32>,
    pub logprobs: Option<LogprobResult>,
}

#[derive(Debug, Deserialize)]
/// This is a dummy struct, it's not actually used.
pub struct LogprobResult {}

#[derive(Debug, Deserialize)]
/// Completion response from OpenAI.
pub struct CompletionResponse<'a> {
    pub id: &'a str,
    pub object: &'a str,
    pub created: i64,
    pub model: &'a str,
    pub choices: heapless::Vec<CompletionChoice, 3>,
    pub usage: Usage,
}
