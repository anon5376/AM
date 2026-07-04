use crate::beval::results::TransportMetadata;
use crate::beval::transport::LlmTransport;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub fn chat(_prompt: &str) -> Result<String> {
    if std::env::var("AM_ENABLE_OLLAMA").ok().as_deref() != Some("1") {
        anyhow::bail!("AM_ENABLE_OLLAMA is not set to 1")
    }
    anyhow::bail!("Ollama client is a v0 seam stub")
}

#[derive(Clone, Debug)]
pub struct OllamaTransport {
    endpoint: String,
    model: String,
    digest: String,
    temperature: f32,
    seed: Option<u64>,
}

impl OllamaTransport {
    pub fn from_env() -> Result<Self> {
        if env::var("AM_ENABLE_OLLAMA").ok().as_deref() != Some("1") {
            anyhow::bail!("live beval transport requires AM_ENABLE_OLLAMA=1");
        }
        let model = env::var("AM_OLLAMA_MODEL").context("live beval requires AM_OLLAMA_MODEL")?;
        let digest =
            env::var("AM_OLLAMA_DIGEST").context("live beval requires AM_OLLAMA_DIGEST")?;
        let endpoint =
            env::var("AM_OLLAMA_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:11434".into());
        let seed = match env::var("AM_OLLAMA_SEED") {
            Ok(value) => Some(value.parse().context("parse AM_OLLAMA_SEED")?),
            Err(_) => Some(0),
        };
        Ok(Self {
            endpoint,
            model,
            digest,
            temperature: 0.0,
            seed,
        })
    }

    fn post_chat(&self, prompt: &str) -> Result<String> {
        let (host, port) = parse_http_endpoint(&self.endpoint)?;
        let mut stream = TcpStream::connect((host.as_str(), port))
            .with_context(|| format!("connect Ollama endpoint {}", self.endpoint))?;
        stream.set_read_timeout(Some(Duration::from_secs(120)))?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let mut options = json!({ "temperature": self.temperature });
        if let Some(seed) = self.seed {
            options["seed"] = json!(seed);
        }
        let body = json!({
            "model": self.model,
            "messages": [{ "role": "user", "content": prompt }],
            "stream": false,
            "options": options
        })
        .to_string();
        let request = format!(
            "POST /api/chat HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(request.as_bytes())?;
        let mut response = String::new();
        stream.read_to_string(&mut response)?;
        parse_http_chat_response(&response)
    }
}

impl LlmTransport for OllamaTransport {
    fn complete(&mut self, prompt: &str) -> Result<String> {
        self.post_chat(prompt)
    }

    fn metadata(&self) -> TransportMetadata {
        TransportMetadata::live(
            self.model.clone(),
            self.digest.clone(),
            self.temperature,
            self.seed,
            self.endpoint.clone(),
        )
    }
}

fn parse_http_endpoint(endpoint: &str) -> Result<(String, u16)> {
    let rest = endpoint
        .strip_prefix("http://")
        .context("Ollama endpoint must use http://")?;
    let host_port = rest.trim_end_matches('/');
    let (host, port) = host_port
        .rsplit_once(':')
        .context("Ollama endpoint must include host:port")?;
    Ok((host.to_string(), port.parse().context("parse Ollama port")?))
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatResponse {
    message: Option<ChatMessage>,
    response: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatMessage {
    content: String,
}

fn parse_http_chat_response(response: &str) -> Result<String> {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .context("Ollama HTTP response missing body")?;
    let status = response.lines().next().unwrap_or_default();
    anyhow::ensure!(
        status.contains(" 200 "),
        "Ollama HTTP request failed: {status}"
    );
    let parsed: ChatResponse = serde_json::from_str(body).context("parse Ollama chat response")?;
    if let Some(message) = parsed.message {
        return Ok(message.content);
    }
    if let Some(response) = parsed.response {
        return Ok(response);
    }
    anyhow::bail!("Ollama chat response missing message.content")
}

#[cfg(test)]
mod tests {
    use super::{parse_http_chat_response, parse_http_endpoint};

    #[test]
    fn endpoint_parser_requires_http_host_port() {
        assert_eq!(
            parse_http_endpoint("http://127.0.0.1:11434").unwrap(),
            ("127.0.0.1".to_string(), 11434)
        );
        assert!(parse_http_endpoint("https://127.0.0.1:11434").is_err());
    }

    #[test]
    fn chat_response_parser_reads_message_content() {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 40\r\n\r\n{\"message\":{\"content\":\"ANSWER: ok\"}}";
        assert_eq!(parse_http_chat_response(response).unwrap(), "ANSWER: ok");
    }
}
