use serde::{Deserialize, Serialize};

/// Configuration for one engine turn. Mirrors the parts of the Mavis
/// `IAgentConfig` / `IModelRef` contract that we actually need.
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfig {
    /// Never accepted from the webview: the key is resolved in Rust from the OS keyring.
    #[serde(skip, default)]
    pub api_key: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub workspace: String,
    #[serde(default)]
    pub full_authorization: bool,
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
}

fn default_base_url() -> String {
    "https://api.deepseek.com".to_string()
}

fn default_model() -> String {
    "deepseek-flash".to_string()
}

fn default_max_turns() -> u32 {
    24
}

impl EngineConfig {
    pub fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
    }
}

/// A conversation message in OpenAI chat-completions shape.
#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OutgoingToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: Some(content.into()),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutgoingToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: OutgoingFunction,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OutgoingFunction {
    pub name: String,
    pub arguments: String,
}

/// Event streamed from the engine to the frontend.
///
/// Maps onto the Mavis runtime contract (`RuntimeEventType`):
/// - `kind = "stream"`        -> `STREAM_RESP`
/// - `kind = "action_required"` -> `ACTION_REQUIRED`
/// - `kind = "status"`        -> `SESSION_STATUS`
/// - `kind = "turn_end"`      -> `TURN_TERMINAL`
/// - `kind = "tool_start"` / `"tool_result"` / `"error"` are presentation-level.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineEvent {
    pub session_id: String,
    pub seq: u64,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<ToolEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl EngineEvent {
    pub fn new(session_id: &str, seq: u64, kind: &str) -> Self {
        Self {
            session_id: session_id.to_string(),
            seq,
            kind: kind.to_string(),
            text: None,
            tool: None,
            action: None,
            status: None,
            stop_reason: None,
            usage: None,
            error: None,
        }
    }
}

/// Something the turn is blocked on. Mirrors the Mavis `ACTION_REQUIRED` contract.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEvent {
    pub id: String,
    /// "question" for now; "permission" is the planned next one.
    pub kind: String,
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolEvent {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageEvent {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
