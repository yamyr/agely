//! Live conversation data — see specs/002-live-codex-agent/data-model.md.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Speaker {
    Agent,
    Person,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptLine {
    pub speaker: Speaker,
    pub text: String,
}

/// Canonical language IDs shared with `site/i18n/core.js`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "zh-Hans")]
    SimplifiedChinese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "da")]
    Danish,
    #[serde(rename = "uk")]
    Ukrainian,
    #[serde(rename = "pl")]
    Polish,
    #[serde(rename = "ro")]
    Romanian,
    #[serde(rename = "nb")]
    NorwegianBokmal,
    #[serde(rename = "fi")]
    Finnish,
    #[serde(rename = "id")]
    Indonesian,
    #[serde(rename = "tl")]
    Tagalog,
    #[serde(rename = "ta")]
    Tamil,
    #[serde(rename = "hi")]
    Hindi,
}

impl Locale {
    pub const fn id(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
            Self::Spanish => "es",
            Self::German => "de",
            Self::Italian => "it",
            Self::SimplifiedChinese => "zh-Hans",
            Self::Korean => "ko",
            Self::Japanese => "ja",
            Self::Danish => "da",
            Self::Ukrainian => "uk",
            Self::Polish => "pl",
            Self::Romanian => "ro",
            Self::NorwegianBokmal => "nb",
            Self::Finnish => "fi",
            Self::Indonesian => "id",
            Self::Tagalog => "tl",
            Self::Tamil => "ta",
            Self::Hindi => "hi",
        }
    }

    pub fn language(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::French => "French",
            Self::Spanish => "Spanish",
            Self::German => "German",
            Self::Italian => "Italian",
            Self::SimplifiedChinese => "Simplified Chinese",
            Self::Korean => "Korean",
            Self::Japanese => "Japanese",
            Self::Danish => "Danish",
            Self::Ukrainian => "Ukrainian",
            Self::Polish => "Polish",
            Self::Romanian => "Romanian",
            Self::NorwegianBokmal => "Norwegian Bokmål",
            Self::Finnish => "Finnish",
            Self::Indonesian => "Indonesian",
            Self::Tagalog => "Tagalog",
            Self::Tamil => "Tamil",
            Self::Hindi => "Hindi",
        }
    }
}

/// Per-conversation instructions shared with `src/demo/live/briefs.ts`.
/// The generated asset also contains a `conversation` identifier, which is ignored here.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentBrief {
    /// Older clients omit this field and retain English responses.
    #[serde(default)]
    pub locale: Locale,
    pub agent_role: String,
    pub goal: String,
    pub background: String,
    /// "Margaret" or "Dev": whose voice the suggested replies use.
    pub reply_as: String,
    pub max_turns: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnRequest {
    pub brief: AgentBrief,
    pub transcript: Vec<TranscriptLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentTurn {
    pub message: String,
    pub options: Vec<String>,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotReady,
    Timeout,
    Failed,
    Malformed,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentError {
    pub code: ErrorCode,
    pub message: String,
}

impl AgentError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusState {
    Ready,
    NotInstalled,
    NotSignedIn,
    ModelUnavailable,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatus {
    pub state: StatusState,
    pub model: Option<String>,
    pub detail: String,
    pub fix: Option<String>,
}
