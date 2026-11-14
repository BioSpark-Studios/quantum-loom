//! Ollama client. Local only: the default endpoint is `http://127.0.0.1:11434`
//! and plain http is all that is compiled in. Calls block, so run them on a
//! worker thread (see [`spawn`]).

use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::sync::mpsc;
use std::time::Duration;

pub const DEFAULT_ENDPOINT: &str = "http://127.0.0.1:11434";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmSettings {
    pub endpoint: String,
    pub model: String,
    /// Seconds to wait for a reply before giving up.
    pub timeout_s: u64,
    pub system: String,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            endpoint: DEFAULT_ENDPOINT.into(),
            model: "llama3.2".into(),
            timeout_s: 120,
            system: String::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LlmError {
    /// Nothing is listening, or the request never completed.
    Unreachable(String),
    /// The server answered with an error (unknown model, bad request …).
    Server(String),
    /// A reply that is not the NDJSON / JSON Ollama sends.
    Malformed(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Unreachable(e) => write!(f, "Ollama is not reachable: {e}"),
            LlmError::Server(e) => write!(f, "Ollama replied with an error: {e}"),
            LlmError::Malformed(e) => write!(f, "unexpected reply: {e}"),
        }
    }
}

fn agent(timeout_s: u64) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(timeout_s.max(1))))
        .http_status_as_error(false)
        .build()
        .into()
}

fn base(s: &LlmSettings) -> String {
    s.endpoint.trim_end_matches('/').to_owned()
}

#[derive(Deserialize)]
struct Tags {
    models: Vec<TagModel>,
}
#[derive(Deserialize)]
struct TagModel {
    name: String,
}

/// Names of the models the server has pulled.
pub fn list_models(s: &LlmSettings) -> Result<Vec<String>, LlmError> {
    let mut r = agent(s.timeout_s.min(10))
        .get(&format!("{}/api/tags", base(s)))
        .call()
        .map_err(|e| LlmError::Unreachable(e.to_string()))?;
    if r.status().as_u16() >= 400 {
        return Err(LlmError::Server(format!("HTTP {}", r.status())));
    }
    let tags: Tags = r
        .body_mut()
        .read_json()
        .map_err(|e| LlmError::Malformed(e.to_string()))?;
    Ok(tags.models.into_iter().map(|m| m.name).collect())
}

#[derive(Deserialize)]
struct Chunk {
    #[serde(default)]
    message: Option<Msg>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    done: bool,
}
#[derive(Deserialize)]
struct Msg {
    content: String,
}

/// Stream a chat completion. `on_token` gets each piece as it arrives; the
/// full text is returned. `/api/chat` with NDJSON streaming.
pub fn chat(
    s: &LlmSettings,
    system: &str,
    prompt: &str,
    mut on_token: impl FnMut(&str),
) -> Result<String, LlmError> {
    let mut messages = vec![];
    let sys = if system.is_empty() {
        s.system.as_str()
    } else {
        system
    };
    if !sys.is_empty() {
        messages.push(serde_json::json!({"role": "system", "content": sys}));
    }
    messages.push(serde_json::json!({"role": "user", "content": prompt}));
    let body = serde_json::json!({"model": s.model, "messages": messages, "stream": true});
    let mut r = agent(s.timeout_s)
        .post(&format!("{}/api/chat", base(s)))
        .send_json(&body)
        .map_err(|e| LlmError::Unreachable(e.to_string()))?;
    if r.status().as_u16() >= 400 {
        let code = r.status().as_u16();
        let text = r.body_mut().read_to_string().unwrap_or_default();
        let msg = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str().map(str::to_owned)))
            .unwrap_or(text);
        return Err(LlmError::Server(format!("HTTP {code}: {msg}")));
    }
    let mut out = String::new();
    let reader = BufReader::new(r.body_mut().as_reader());
    for line in reader.lines() {
        let line = line.map_err(|e| LlmError::Unreachable(e.to_string()))?;
        if line.trim().is_empty() {
            continue;
        }
        let c: Chunk =
            serde_json::from_str(&line).map_err(|e| LlmError::Malformed(format!("{e}: {line}")))?;
        if let Some(e) = c.error {
            return Err(LlmError::Server(e));
        }
        if let Some(m) = c.message {
            on_token(&m.content);
            out.push_str(&m.content);
        }
        if c.done {
            break;
        }
    }
    Ok(out)
}

/// Run `job` on a worker thread; the result arrives on the returned channel.
pub fn spawn<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
    wake: impl Fn() + Send + 'static,
) -> mpsc::Receiver<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(job());
        wake();
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// One-shot HTTP server answering with `status` and `body`.
    fn serve(status: &'static str, body: &'static str) -> String {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = l.local_addr().unwrap();
        std::thread::spawn(move || {
            if let Ok((mut s, _)) = l.accept() {
                // Drain the whole request (headers + body) before replying,
                // or closing with unread data resets the connection.
                let mut req = vec![];
                let mut buf = [0u8; 4096];
                loop {
                    let n = s.read(&mut buf).unwrap_or(0);
                    req.extend_from_slice(&buf[..n]);
                    let text = String::from_utf8_lossy(&req).to_lowercase();
                    let Some(h) = text.find("\r\n\r\n") else {
                        if n == 0 {
                            break;
                        }
                        continue;
                    };
                    let want = text
                        .lines()
                        .find_map(|l| {
                            l.strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if req.len() >= h + 4 + want || n == 0 {
                        break;
                    }
                }
                let _ = write!(s, "HTTP/1.1 {status}\r\nContent-Type: application/x-ndjson\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
            }
        });
        format!("http://{addr}")
    }

    fn settings(endpoint: String) -> LlmSettings {
        LlmSettings {
            endpoint,
            timeout_s: 5,
            ..Default::default()
        }
    }

    #[test]
    fn streams_tokens_in_order() {
        let ep = serve("200 OK", "{\"message\":{\"content\":\"Hel\"},\"done\":false}\n{\"message\":{\"content\":\"lo\"},\"done\":false}\n{\"done\":true}\n");
        let mut got = vec![];
        let text = chat(&settings(ep), "", "hi", |t| got.push(t.to_owned())).unwrap();
        assert_eq!(text, "Hello");
        assert_eq!(got, vec!["Hel", "lo"]);
    }

    #[test]
    fn lists_models() {
        let ep = serve(
            "200 OK",
            "{\"models\":[{\"name\":\"llama3.2:latest\"},{\"name\":\"qwen2.5:7b\"}]}",
        );
        assert_eq!(
            list_models(&settings(ep)).unwrap(),
            vec!["llama3.2:latest", "qwen2.5:7b"]
        );
    }

    #[test]
    fn server_errors_are_reported() {
        let ep = serve("404 Not Found", "{\"error\":\"model 'nope' not found\"}");
        match chat(&settings(ep), "", "hi", |_| {}) {
            Err(LlmError::Server(m)) => assert!(m.contains("not found"), "{m}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn mid_stream_error_surfaces() {
        let ep = serve(
            "200 OK",
            "{\"message\":{\"content\":\"a\"},\"done\":false}\n{\"error\":\"out of memory\"}\n",
        );
        assert_eq!(
            chat(&settings(ep), "", "hi", |_| {}),
            Err(LlmError::Server("out of memory".into()))
        );
    }

    #[test]
    fn nothing_listening_is_unreachable() {
        // port 1 is never an Ollama
        let r = chat(&settings("http://127.0.0.1:1".into()), "", "hi", |_| {});
        assert!(matches!(r, Err(LlmError::Unreachable(_))), "{r:?}");
    }

    #[test]
    fn worker_thread_delivers() {
        let rx = spawn(|| 41 + 1, || {});
        assert_eq!(rx.recv_timeout(Duration::from_secs(2)).unwrap(), 42);
    }
}
