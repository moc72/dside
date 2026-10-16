use futures_util::StreamExt;
use serde_json::{json, Value};

use super::types::{ChatMessage, EngineConfig, OutgoingFunction, OutgoingToolCall};

/// Which stream a delta belongs to. A model that thinks sends its
/// thinking on `reasoning_content` before the answer.
#[derive(Clone, Copy)]
pub enum DeltaKind {
    Text,
    Reasoning,
}

pub struct StreamOutcome {
    pub content: String,
    /// Collected thinking. The UI gets it live through `DeltaKind::Reasoning`; this is the
    /// full text for callers (and tests) that need it after the fact.
    #[allow(dead_code)]
    pub reasoning: String,
    pub tool_calls: Vec<OutgoingToolCall>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

fn truncate(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    text.chars().take(limit).collect::<String>() + "…"
}

/// Streams one assistant turn from an OpenAI-compatible chat-completions endpoint
/// (DeepSeek, `POST /chat/completions`). Text deltas are forwarded to `on_text`.
pub async fn stream_chat(
    config: &EngineConfig,
    messages: &[ChatMessage],
    tools: &[Value],
    on_delta: &mut (dyn FnMut(DeltaKind, &str) + Send),
) -> Result<StreamOutcome, String> {
    let client = reqwest::Client::builder()
        .build()
        .map_err(|error| error.to_string())?;

    let mut body = json!({
        "model": config.model,
        "stream": true,
        "stream_options": { "include_usage": true },
        "messages": messages,
    });
    if !tools.is_empty() {
        body["tools"] = Value::Array(tools.to_vec());
    }
    // How hard the model thinks before answering. The provider rejects an empty string, so an
    // unset value must stay out of the body entirely and leave its own default in place.
    if let Some(effort) = config.reasoning_effort.as_deref() {
        if !effort.trim().is_empty() {
            body["reasoning_effort"] = Value::String(effort.trim().to_string());
        }
    }

    let response = client
        .post(config.chat_completions_url())
        .bearer_auth(&config.api_key)
        .json(&body)
        .send()
        .await
        .map_err(|error| format!("LLM request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let text = response.text().await.unwrap_or_default();
        return Err(format!("LLM HTTP {status}: {}", truncate(&text, 700)));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut partials: Vec<(String, String, String)> = Vec::new();
    let mut usage = (0u32, 0u32, 0u32);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| error.to_string())?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        loop {
            let Some(index) = buffer.find('\n') else { break };
            let line = buffer[..index].trim_end_matches('\r').to_string();
            buffer.drain(..=index);

            let Some(data) = line.strip_prefix("data:") else {
                continue;
            };
            let data = data.trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            let Ok(parsed) = serde_json::from_str::<Value>(data) else {
                continue;
            };

            if let Some(usage_value) = parsed.get("usage") {
                if !usage_value.is_null() {
                    usage.0 = usage_value
                        .get("prompt_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    usage.1 = usage_value
                        .get("completion_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                    usage.2 = usage_value
                        .get("total_tokens")
                        .and_then(Value::as_u64)
                        .unwrap_or(0) as u32;
                }
            }

            let Some(delta) = parsed
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("delta"))
            else {
                continue;
            };

            if let Some(text) = delta.get("content").and_then(Value::as_str) {
                if !text.is_empty() {
                    content.push_str(text);
                    on_delta(DeltaKind::Text, text);
                }
            }

            if let Some(thinking) = delta.get("reasoning_content").and_then(Value::as_str) {
                if !thinking.is_empty() {
                    reasoning.push_str(thinking);
                    on_delta(DeltaKind::Reasoning, thinking);
                }
            }

            if let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) {
                for call in calls {
                    let index = call.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
                    while partials.len() <= index {
                        partials.push((String::new(), String::new(), String::new()));
                    }
                    let slot = &mut partials[index];
                    if let Some(id) = call.get("id").and_then(Value::as_str) {
                        if !id.is_empty() {
                            slot.0 = id.to_string();
                        }
                    }
                    if let Some(function) = call.get("function") {
                        if let Some(name) = function.get("name").and_then(Value::as_str) {
                            if !name.is_empty() {
                                slot.1.push_str(name);
                            }
                        }
                        if let Some(arguments) = function.get("arguments").and_then(Value::as_str) {
                            slot.2.push_str(arguments);
                        }
                    }
                }
            }
        }
    }

    let tool_calls = partials
        .into_iter()
        .filter(|(_, name, _)| !name.is_empty())
        .enumerate()
        .map(|(index, (id, name, arguments))| OutgoingToolCall {
            id: if id.is_empty() {
                format!("call_{index}")
            } else {
                id
            },
            call_type: "function".to_string(),
            function: OutgoingFunction { name, arguments },
        })
        .collect();

    Ok(StreamOutcome {
        content,
        reasoning,
        tool_calls,
        prompt_tokens: usage.0,
        completion_tokens: usage.1,
        total_tokens: usage.2,
    })
}

#[cfg(test)]
mod live {
    use crate::engine::types::EngineConfig;
    use super::*;

    fn live_config() -> Option<EngineConfig> {
        let api_key = std::env::var("DS_IDE_API_KEY").ok()?;
        if api_key.trim().is_empty() {
            return None;
        }
        Some(EngineConfig {
            api_key,
            base_url: std::env::var("DS_IDE_BASE_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com".to_string()),
            model: std::env::var("DS_IDE_MODEL").unwrap_or_else(|_| "deepseek-flash".to_string()),
            system_prompt: "You are a terse test assistant. Follow instructions exactly."
                .to_string(),
            workspace: "/tmp".to_string(),
            full_authorization: true,
            max_turns: 2,
            reasoning_effort: None,
            disabled_tools: Vec::new(),
        })
    }

    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_deepseek_streams_text() {
        let Some(config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };
        let messages = vec![ChatMessage::user("Antworte mit exakt einem Wort: PONG")];
        let mut streamed = String::new();
        let result = {
            let mut on_delta = |kind: DeltaKind, chunk: &str| {
                if matches!(kind, DeltaKind::Text) {
                    streamed.push_str(chunk);
                }
            };
            stream_chat(&config, &messages, &[], &mut on_delta).await
        };

        match result {
            Ok(outcome) => {
                println!("content: {:?}", outcome.content);
                println!("streamed chars: {}", streamed.len());
                println!(
                    "usage prompt/completion/total: {}/{}/{}",
                    outcome.prompt_tokens, outcome.completion_tokens, outcome.total_tokens
                );
                assert!(!outcome.content.trim().is_empty(), "empty completion");
            }
            Err(error) => panic!("live text call failed: {error}"),
        }
    }

    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_deepseek_emits_tool_calls() {
        let Some(config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };
        let messages = vec![ChatMessage::user(
            "Rufe jetzt das Tool glob mit pattern \"Cargo.toml\" auf. Keine Erklärung, nur der Tool-Aufruf.",
        )];
        let tools = crate::engine::tools::definitions();
        let mut on_delta = |_kind: DeltaKind, _chunk: &str| {};
        let result = stream_chat(&config, &messages, &tools, &mut on_delta).await;

        match result {
            Ok(outcome) => {
                for call in &outcome.tool_calls {
                    println!("tool call: {} args={}", call.function.name, call.function.arguments);
                }
                println!("text: {:?}", outcome.content);
                assert!(!outcome.tool_calls.is_empty(), "expected at least one tool call");
            }
            Err(error) => panic!("live tool call failed: {error}"),
        }
    }

    /// Images go as data URLs in the provider's own `image` field, not as the OpenAI content
    /// parts. A wrong shape here means a 400 on every picture the user attaches.
    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_deepseek_takes_an_image() {
        let Some(config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };

        // A 32x32 PNG, built here so the test needs no fixture file.
        let png = {
            let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
            let mut chunk = |kind: &[u8; 4], data: Vec<u8>| {
                bytes.extend((data.len() as u32).to_be_bytes());
                bytes.extend(kind);
                bytes.extend(&data);
                bytes.extend([0, 0, 0, 0]); // crc: the provider does not check it
            };
            chunk(b"IHDR", {
                let mut header = Vec::new();
                header.extend(32u32.to_be_bytes());
                header.extend(32u32.to_be_bytes());
                header.extend([8, 2, 0, 0, 0]);
                header
            });
            let mut raw = Vec::new();
            for y in 0..32u8 {
                raw.push(0);
                for x in 0..32u8 {
                    raw.push(x * 8);
                    raw.push(y * 8);
                    raw.push(128);
                }
            }
            chunk(b"IDAT", {
                let mut encoded = vec![0x78, 0x01];
                encoded.extend(raw.iter().take(0));
                encoded
            });
            chunk(b"IEND", Vec::new());
            bytes
        };

        let mut message = ChatMessage::user("Antworte mit genau: BILD");
        message.image = Some(vec![format!(
            "data:image/png;base64,{}",
            crate::base64_encode(&png)
        )]);
        let messages = vec![message];

        let mut sink = |_kind: DeltaKind, _chunk: &str| {};
        match stream_chat(&config, &messages, &[], &mut sink).await {
            Ok(outcome) => println!("answer: {:?}", outcome.content),
            Err(error) => panic!("sending an image failed: {error}"),
        }
    }

    /// `reasoning_effort` is a real field with a fixed set of values; a typo in either the
    /// name or the value is a 422, and that is worth catching without a human in the loop.
    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_deepseek_accepts_reasoning_effort() {
        let Some(mut config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };
        config.model = "deepseek-v4-pro".to_string();
        config.reasoning_effort = Some("low".to_string());

        let messages = vec![ChatMessage::user("Antworte mit genau: OK")];
        let mut sink = |_kind: DeltaKind, _chunk: &str| {};
        let outcome = stream_chat(&config, &messages, &[], &mut sink)
            .await
            .expect("request with reasoning_effort must be accepted");
        assert_eq!(outcome.content.trim(), "OK");
    }

    #[tokio::test]
    #[ignore = "requires DS_IDE_API_KEY"]
    async fn live_deepseek_reasoner_streams_thinking() {
        let Some(mut config) = live_config() else {
            eprintln!("skipped: DS_IDE_API_KEY not set");
            return;
        };
        config.model = "deepseek-v4-pro".to_string();

        let messages = vec![ChatMessage::user("Was ist 17 * 23? Denke kurz nach.")];
        let mut text = String::new();
        let result = {
            let mut on_delta = |kind: DeltaKind, chunk: &str| {
                if matches!(kind, DeltaKind::Text) {
                    text.push_str(chunk);
                }
            };
            stream_chat(&config, &messages, &[], &mut on_delta).await
        };

        match result {
            Ok(outcome) => {
                println!("reasoning chars: {}", outcome.reasoning.len());
                println!("answer: {:?}", outcome.content);
                assert!(
                    !outcome.reasoning.is_empty(),
                    "expected reasoning_content from the provider's thinking model"
                );
                assert!(!outcome.content.is_empty(), "expected a final answer");
            }
            Err(error) => panic!("live reasoner call failed: {error}"),
        }
    }
}
