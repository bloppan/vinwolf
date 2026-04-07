use std::collections::HashMap;
use std::fmt;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

use crate::serde::{Deserialize, Serialize, Value};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum RpcError {
    Bind(String),
    Transport(String),
    InvalidRequest(String),
    MethodNotFound(String),
    InternalError(String),
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind(s)           => write!(f, "bind error: {}", s),
            Self::Transport(s)      => write!(f, "transport error: {}", s),
            Self::InvalidRequest(s) => write!(f, "invalid request: {}", s),
            Self::MethodNotFound(s) => write!(f, "method not found: {}", s),
            Self::InternalError(s)  => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for RpcError {}

// JSON-RPC error codes (spec)
pub const CODE_PARSE_ERROR:      i64 = -32700;
pub const CODE_INVALID_REQUEST:  i64 = -32600;
pub const CODE_METHOD_NOT_FOUND: i64 = -32601;
pub const CODE_INTERNAL_ERROR:   i64 = -32603;

// ---------------------------------------------------------------------------
// JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum RpcId {
    Number(u64),
    Str(String),
    Null,
}

impl Serialize for RpcId {
    fn to_value(&self) -> Value {
        match self {
            Self::Number(n) => n.to_value(),
            Self::Str(s)    => s.to_value(),
            Self::Null      => Value::Null,
        }
    }
}

impl Deserialize for RpcId {
    fn from_value(v: &Value) -> Result<Self, String> {
        match v {
            Value::Number(s) => s.parse::<u64>().map(Self::Number).or_else(|_| Ok(Self::Str(s.clone()))),
            Value::String(s) => Ok(Self::Str(s.clone())),
            Value::Null      => Ok(Self::Null),
            _ => Err("expected id (number, string or null)".into()),
        }
    }
}

struct RpcRequest {
    id:     RpcId,
    method: String,
    params: Vec<Value>,
}

fn parse_request(v: &Value) -> Result<RpcRequest, (i64, String)> {
    let o = match v {
        Value::Object(o) => o,
        _ => return Err((CODE_INVALID_REQUEST, "request must be an object".into())),
    };

    let jsonrpc = o.get("jsonrpc")
        .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
        .unwrap_or("");
    if jsonrpc != "2.0" {
        return Err((CODE_INVALID_REQUEST, "jsonrpc must be \"2.0\"".into()));
    }

    let id = match o.get("id") {
        Some(v) => RpcId::from_value(v).unwrap_or(RpcId::Null),
        None    => RpcId::Null,
    };

    let method = match o.get("method") {
        Some(Value::String(s)) => s.clone(),
        _ => return Err((CODE_INVALID_REQUEST, "missing or invalid method".into())),
    };

    let params = match o.get("params") {
        Some(Value::Array(a)) => a.clone(),
        Some(Value::Null) | None => vec![],
        _ => return Err((CODE_INVALID_REQUEST, "params must be an array".into())),
    };

    Ok(RpcRequest { id, method, params })
}

fn build_success(id: &RpcId, result: Value) -> String {
    let mut m = HashMap::new();
    m.insert("jsonrpc".into(), Value::String("2.0".into()));
    m.insert("id".into(),      id.to_value());
    m.insert("result".into(),  result);
    crate::serde::stringify(&Value::Object(m))
}

fn build_error(id: &RpcId, code: i64, message: String) -> String {
    let mut err = HashMap::new();
    err.insert("code".into(),    Value::Number(code.to_string()));
    err.insert("message".into(), Value::String(message));

    let mut m = HashMap::new();
    m.insert("jsonrpc".into(), Value::String("2.0".into()));
    m.insert("id".into(),      id.to_value());
    m.insert("error".into(),   Value::Object(err));
    crate::serde::stringify(&Value::Object(m))
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

fn read_http_request(stream: &mut TcpStream) -> Result<String, RpcError> {
    let mut raw = Vec::new();
    let mut buf = [0u8; 4096];

    // Read until we have the full headers + body
    let separator = b"\r\n\r\n";
    let header_end;
    loop {
        let n = stream.read(&mut buf).map_err(|e| RpcError::Transport(e.to_string()))?;
        if n == 0 {
            return Err(RpcError::Transport("connection closed".into()));
        }
        raw.extend_from_slice(&buf[..n]);

        if let Some(pos) = find_subsequence(&raw, separator) {
            header_end = pos + separator.len();
            break;
        }

        if raw.len() > 64 * 1024 {
            return Err(RpcError::InvalidRequest("headers too large".into()));
        }
    }

    // Parse Content-Length from headers
    let headers = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| RpcError::InvalidRequest("non-utf8 headers".into()))?;

    let content_length = headers.lines()
        .find_map(|line| {
            let l = line.to_ascii_lowercase();
            l.strip_prefix("content-length:").map(|v| v.trim().to_string())
        })
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0);

    // Read remaining body bytes if not already buffered
    while raw.len() < header_end + content_length {
        let n = stream.read(&mut buf).map_err(|e| RpcError::Transport(e.to_string()))?;
        if n == 0 { break; }
        raw.extend_from_slice(&buf[..n]);
    }

    let body = std::str::from_utf8(&raw[header_end..header_end + content_length])
        .map_err(|_| RpcError::InvalidRequest("non-utf8 body".into()))?
        .to_string();

    Ok(body)
}

fn write_http_response(stream: &mut TcpStream, body: &str) -> Result<(), RpcError> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(), body
    );
    stream.write_all(response.as_bytes()).map_err(|e| RpcError::Transport(e.to_string()))
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

// ---------------------------------------------------------------------------
// Connection handler
// ---------------------------------------------------------------------------

fn handle_connection<H>(mut stream: TcpStream, handler: &H)
where
    H: Fn(&str, Vec<Value>) -> Result<Value, (i64, String)>,
{
    let timeout = std::time::Duration::from_secs(10);
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let body = match read_http_request(&mut stream) {
        Ok(b) => b,
        Err(_) => return,
    };

    let response_body = match crate::serde::parse(&body) {
        Err(_) => build_error(&RpcId::Null, CODE_PARSE_ERROR, "parse error".into()),
        Ok(v)  => match parse_request(&v) {
            Err((code, msg)) => build_error(&RpcId::Null, code, msg),
            Ok(req) => match handler(&req.method, req.params) {
                Ok(result) => build_success(&req.id, result),
                Err((code, msg)) => build_error(&req.id, code, msg),
            },
        },
    };

    let _ = write_http_response(&mut stream, &response_body);
}

// ---------------------------------------------------------------------------
// RpcServer
// ---------------------------------------------------------------------------

pub struct RpcServer {
    listener: TcpListener,
}

impl RpcServer {
    pub fn bind(port: u16) -> Result<Self, RpcError> {
        let listener = TcpListener::bind(("0.0.0.0", port))
            .map_err(|e| RpcError::Bind(e.to_string()))?;
        Ok(Self { listener })
    }

    /// Blocking. Spawn in a dedicated thread.
    /// `handler(method, params) -> Ok(result_value) | Err((code, message))`
    pub fn run<H>(self, handler: H)
    where
        H: Fn(&str, Vec<Value>) -> Result<Value, (i64, String)> + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let h = handler.clone();
                    std::thread::spawn(move || handle_connection(stream, h.as_ref()));
                }
                Err(e) => {
                    crate::log::__private_log(
                        crate::log::Level::Error,
                        "tools::rpc",
                        format_args!("accept error: {}", e),
                    );
                }
            }
        }
    }
}

// Re-export error codes for use by handlers
pub mod codes {
    pub use super::{CODE_INTERNAL_ERROR, CODE_METHOD_NOT_FOUND};
}
