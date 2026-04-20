use crate::prompt;
use futures_util::{Stream, StreamExt};
use paladin_core::error::{Error, Result};
use paladin_core::{FeatureBundle, TradeSignal};
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::time::Duration;
use tracing::debug;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

impl ChatMessage {
    pub fn system(s: impl Into<String>) -> Self {
        Self { role: Role::System, content: s.into() }
    }
    pub fn user(s: impl Into<String>) -> Self {
        Self { role: Role::User, content: s.into() }
    }
    pub fn assistant(s: impl Into<String>) -> Self {
        Self { role: Role::Assistant, content: s.into() }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessageOwned,
}

#[derive(Deserialize)]
struct ChatMessageOwned {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
}

#[derive(Deserialize)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
}

impl Client {
    pub fn new(base_url: impl Into<String>, api_key: Option<String>, model: impl Into<String>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| Error::Other(format!("http client: {e}")))?;
        Ok(Self { http, base_url: base_url.into(), api_key, model: model.into() })
    }

    fn url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        format!("{base}{path}")
    }

    fn auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(key) = &self.api_key {
            req.bearer_auth(key)
        } else {
            req
        }
    }

    /// Non-streaming chat. Returns full assistant content.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        let body = ChatRequest {
            model: &self.model,
            messages,
            temperature: Some(0.4),
            response_format: None,
            stream: false,
        };
        let resp = self
            .auth(self.http.post(self.url("/chat/completions")).json(&body))
            .send()
            .await
            .map_err(|e| Error::Other(format!("http: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Other(format!("api {status}: {text}")));
        }
        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| Error::Other(format!("decode: {e}")))?;
        Ok(parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default())
    }

    /// Streaming chat. Yields incremental content chunks.
    pub async fn chat_stream(
        &self,
        messages: &[ChatMessage],
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let body = ChatRequest {
            model: &self.model,
            messages,
            temperature: Some(0.4),
            response_format: None,
            stream: true,
        };
        let resp = self
            .auth(self.http.post(self.url("/chat/completions")).json(&body))
            .send()
            .await
            .map_err(|e| Error::Other(format!("http: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Other(format!("api {status}: {text}")));
        }
        let bytes_stream = resp.bytes_stream();
        let mapped = bytes_stream.map(|chunk| {
            chunk.map_err(|e| Error::Other(format!("stream: {e}")))
        });

        let stream = async_stream::try_stream(mapped);
        Ok(stream)
    }

    pub async fn signal(&self, features: &FeatureBundle) -> Result<TradeSignal> {
        let msgs = vec![
            ChatMessage::system(prompt::SIGNAL_SYSTEM_PROMPT),
            ChatMessage::user(prompt::signal_user_prompt(features)),
        ];
        let body = ChatRequest {
            model: &self.model,
            messages: &msgs,
            temperature: Some(0.2),
            response_format: Some(serde_json::json!({"type": "json_object"})),
            stream: false,
        };
        let resp = self
            .auth(self.http.post(self.url("/chat/completions")).json(&body))
            .send()
            .await
            .map_err(|e| Error::Other(format!("http: {e}")))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(Error::Other(format!("api {status}: {text}")));
        }
        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| Error::Other(format!("decode: {e}")))?;
        let content = parsed
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();
        debug!("signal raw: {}", content);
        let signal: TradeSignal = serde_json::from_str(&content)
            .map_err(|e| Error::Other(format!("parse signal json: {e}; raw={content}")))?;
        Ok(signal)
    }
}

mod async_stream {
    use super::*;
    use bytes::Bytes;
    use futures_util::stream::unfold;

    pub fn try_stream<S>(inner: S) -> Pin<Box<dyn Stream<Item = Result<String>> + Send>>
    where
        S: Stream<Item = Result<Bytes>> + Send + 'static,
    {
        let state = (Box::pin(inner), String::new(), false);
        let s = unfold(state, |(mut inner, mut buf, mut done)| async move {
            if done {
                return None;
            }
            loop {
                if let Some(idx) = buf.find("\n\n") {
                    let event: String = buf.drain(..idx + 2).collect();
                    if let Some(payload) = parse_sse(&event) {
                        if payload == "[DONE]" {
                            done = true;
                            return None;
                        }
                        match serde_json::from_str::<StreamChunk>(&payload) {
                            Ok(chunk) => {
                                if let Some(c) = chunk
                                    .choices
                                    .into_iter()
                                    .next()
                                    .and_then(|c| c.delta.content)
                                {
                                    if !c.is_empty() {
                                        return Some((Ok(c), (inner, buf, done)));
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    continue;
                }
                match inner.as_mut().next().await {
                    Some(Ok(bytes)) => {
                        buf.push_str(&String::from_utf8_lossy(&bytes));
                    }
                    Some(Err(e)) => return Some((Err(e), (inner, buf, true))),
                    None => return None,
                }
            }
        });
        Box::pin(s)
    }

    fn parse_sse(event: &str) -> Option<String> {
        let mut out = String::new();
        for line in event.lines() {
            if let Some(rest) = line.strip_prefix("data:") {
                out.push_str(rest.trim());
            }
        }
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }
}
