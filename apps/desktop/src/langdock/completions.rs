use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::ops::ControlFlow;
use thiserror::Error;

use super::client::LangDockClient;

/// LangDock chat message for `POST /agent/v1/chat/completions`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionMessage {
    pub role: String,
    pub content: String,
}

/// Temporary agent envelope (model + system instructions) per sprint refinement.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionAgent {
    pub model: String,
    pub instructions: String,
}

/// Request body for streaming LangDock agent completions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompletionRequestBody {
    pub agent: CompletionAgent,
    pub messages: Vec<CompletionMessage>,
    pub stream: bool,
}

#[derive(Debug, Error)]
pub enum CompletionStreamError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("LangDock API error {status}: {message}")]
    Api { status: u16, message: String },
    #[error("stream cancelled")]
    Cancelled,
}

/// Build a streaming completion request (`stream: true`).
pub fn build_completion_request(
    model: impl Into<String>,
    instructions: impl Into<String>,
    messages: Vec<CompletionMessage>,
) -> CompletionRequestBody {
    CompletionRequestBody {
        agent: CompletionAgent {
            model: model.into(),
            instructions: instructions.into(),
        },
        messages,
        stream: true,
    }
}

/// Parse one Vercel AI SDK data-stream line and extract text from `0:"..."` deltas.
///
/// See <https://ai-sdk.dev/docs/ai-sdk-ui/stream-protocol> (legacy `TYPE_ID:CONTENT` format).
pub fn parse_vercel_text_delta_line(line: &str) -> Option<String> {
    let line = line.trim_end_matches('\r').trim();
    if !line.starts_with("0:") {
        return None;
    }
    serde_json::from_str(line.get(2..)?.trim()).ok()
}

/// Stream completion deltas from LangDock; invoke `on_delta` for each text chunk.
///
/// Return `ControlFlow::Break(())` from `on_delta` to abort the HTTP body (stop run).
pub async fn stream_completion<F>(
    http: &Client,
    base_url: &str,
    api_key: &str,
    body: &CompletionRequestBody,
    mut on_delta: F,
) -> Result<(), CompletionStreamError>
where
    F: FnMut(&str) -> ControlFlow<()>,
{
    let url = format!(
        "{}/agent/v1/chat/completions",
        base_url.trim_end_matches('/')
    );

    let response = http
        .post(&url)
        .bearer_auth(api_key)
        .json(body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let message = response.text().await.unwrap_or_default();
        return Err(CompletionStreamError::Api {
            status: status.as_u16(),
            message,
        });
    }

    let mut byte_stream = response.bytes_stream();
    let mut buffer = String::new();

    while let Some(chunk) = byte_stream.next().await {
        let chunk = chunk?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_idx) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline_idx).collect();
            if let Some(delta) = parse_vercel_text_delta_line(&line) {
                if on_delta(&delta).is_break() {
                    return Err(CompletionStreamError::Cancelled);
                }
            }
        }
    }

    if !buffer.is_empty() {
        if let Some(delta) = parse_vercel_text_delta_line(&buffer) {
            if on_delta(&delta).is_break() {
                return Err(CompletionStreamError::Cancelled);
            }
        }
    }

    Ok(())
}

impl LangDockClient {
    /// Stream completion deltas using this client's HTTP pool.
    pub async fn stream_completion<F>(
        &self,
        base_url: &str,
        api_key: &str,
        body: &CompletionRequestBody,
        on_delta: F,
    ) -> Result<(), CompletionStreamError>
    where
        F: FnMut(&str) -> ControlFlow<()>,
    {
        stream_completion(
            self.http_client(),
            base_url,
            api_key,
            body,
            on_delta,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text_delta_lines() {
        assert_eq!(
            parse_vercel_text_delta_line(r#"0:"Hello""#),
            Some("Hello".into())
        );
        assert_eq!(
            parse_vercel_text_delta_line(r#"0:" is an""#),
            Some(" is an".into())
        );
        assert_eq!(
            parse_vercel_text_delta_line(r#"0:"example.\"""#),
            Some(r#"example.""#.into())
        );
        assert_eq!(
            parse_vercel_text_delta_line("0:\"quoted\"\r\n"),
            Some("quoted".into())
        );
    }

    #[test]
    fn ignores_non_text_stream_lines() {
        assert_eq!(
            parse_vercel_text_delta_line(r#"e:{"finishReason":"stop"}"#),
            None
        );
        assert_eq!(parse_vercel_text_delta_line("2:[1,2]"), None);
        assert_eq!(parse_vercel_text_delta_line(""), None);
        assert_eq!(parse_vercel_text_delta_line("not-a-stream-line"), None);
    }

    #[test]
    fn build_request_sets_stream_and_agent() {
        let body = build_completion_request(
            "mock-model",
            "You are helpful.",
            vec![CompletionMessage {
                role: "user".into(),
                content: "Hi".into(),
            }],
        );
        assert!(body.stream);
        assert_eq!(body.agent.model, "mock-model");
        assert_eq!(body.agent.instructions, "You are helpful.");
        assert_eq!(body.messages.len(), 1);
    }
}
