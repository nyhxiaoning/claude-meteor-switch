use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key_enc: String,
    pub protocol: Protocol,
    pub model_mapping: Option<String>,
    pub auth_header: AuthHeader,
    pub keyword: String,
    pub enabled: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Anthropic,
    OpenAI,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthHeader {
    ApiKey,
    Bearer,
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Anthropic => "anthropic",
            Protocol::OpenAI => "openai",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "anthropic" => Some(Protocol::Anthropic),
            "openai" => Some(Protocol::OpenAI),
            _ => None,
        }
    }
}

impl AuthHeader {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuthHeader::ApiKey => "x-api-key",
            AuthHeader::Bearer => "bearer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "x-api-key" => Some(AuthHeader::ApiKey),
            "bearer" => Some(AuthHeader::Bearer),
            _ => None,
        }
    }
}
