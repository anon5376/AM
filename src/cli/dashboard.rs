use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const OLLAMA_API_BASE: &str = "https://ollama.com/api";
const MAX_DASHBOARD_REQUEST_BYTES: usize = 1_048_576;

pub fn serve_dashboard(host: &str, port: u16) -> Result<()> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("dashboard");
    let root = root
        .canonicalize()
        .with_context(|| format!("dashboard root not found at {}", root.display()))?;
    let listener =
        TcpListener::bind((host, port)).with_context(|| format!("bind {host}:{port}"))?;
    println!("dashboard listening at http://{host}:{port}/");
    for stream in listener.incoming() {
        let mut stream = stream.context("accept dashboard connection")?;
        if let Err(err) = handle_connection(&mut stream, &root) {
            let body = format!("dashboard error: {err}\n");
            write_response(
                &mut stream,
                500,
                "text/plain; charset=utf-8",
                body.as_bytes(),
            )?;
        }
    }
    Ok(())
}

fn handle_connection(stream: &mut impl ReadWrite, root: &Path) -> Result<()> {
    let request = read_request(stream)?;
    if request.target.starts_with("/dashboard-api/") {
        return handle_dashboard_api(stream, &request);
    }

    if request.method != "GET" {
        write_response(
            stream,
            405,
            "text/plain; charset=utf-8",
            b"method not allowed\n",
        )?;
        return Ok(());
    }

    let path = request_path(&request.target);
    let requested = root.join(path);
    let file = requested
        .canonicalize()
        .unwrap_or_else(|_| root.join("__missing__"));
    if !file.starts_with(root) || !file.is_file() {
        write_response(stream, 404, "text/plain; charset=utf-8", b"not found\n")?;
        return Ok(());
    }

    let bytes = fs::read(&file).with_context(|| format!("read {}", file.display()))?;
    write_response(stream, 200, mime_type(&file), &bytes)
}

fn read_request(stream: &mut impl Read) -> Result<HttpRequest> {
    let mut buffer = [0_u8; 8192];
    let read = stream.read(&mut buffer).context("read dashboard request")?;
    if read == 0 {
        bail!("empty dashboard request");
    }
    let mut bytes = buffer[..read].to_vec();
    while find_header_end(&bytes).is_none() {
        if bytes.len() >= MAX_DASHBOARD_REQUEST_BYTES {
            bail!("dashboard request too large");
        }
        let read = stream
            .read(&mut buffer)
            .context("read dashboard request headers")?;
        if read == 0 {
            bail!("incomplete dashboard request headers");
        }
        bytes.extend_from_slice(&buffer[..read]);
    }

    let header_end = find_header_end(&bytes).expect("checked above");
    let header_text = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let mut lines = header_text.lines();
    let Some(first_line) = lines.next() else {
        bail!("bad dashboard request");
    };
    let parts = first_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2 {
        bail!("bad dashboard request line");
    }
    let mut headers = BTreeMap::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }
    let content_length = headers
        .get("content-length")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    if content_length > MAX_DASHBOARD_REQUEST_BYTES {
        bail!("dashboard request body too large");
    }

    let body_start = header_end + 4;
    while bytes.len().saturating_sub(body_start) < content_length {
        let read = stream
            .read(&mut buffer)
            .context("read dashboard request body")?;
        if read == 0 {
            bail!("incomplete dashboard request body");
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    let body = bytes[body_start..body_start + content_length].to_vec();

    Ok(HttpRequest {
        method: parts[0].to_string(),
        target: parts[1].to_string(),
        headers,
        body,
    })
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn handle_dashboard_api(stream: &mut impl Write, request: &HttpRequest) -> Result<()> {
    let path = request.target.split('?').next().unwrap_or("");
    match (request.method.as_str(), path) {
        ("GET", "/dashboard-api/ollama/tags") => proxy_ollama_tags(stream, request),
        ("POST", "/dashboard-api/ollama/chat") => proxy_ollama_chat(stream, request),
        ("OPTIONS", _) => write_response(stream, 204, "text/plain; charset=utf-8", b""),
        _ => {
            write_response(stream, 404, "text/plain; charset=utf-8", b"not found\n")?;
            Ok(())
        }
    }
}

fn proxy_ollama_tags(stream: &mut impl Write, request: &HttpRequest) -> Result<()> {
    let base = ollama_base(request)?;
    let response = run_curl_json(
        "GET",
        &format!("{base}/tags"),
        request.headers.get("authorization").map(String::as_str),
        None,
    )?;
    write_proxy_response(stream, response)
}

fn proxy_ollama_chat(stream: &mut impl Write, request: &HttpRequest) -> Result<()> {
    let base = ollama_base(request)?;
    let response = run_curl_json(
        "POST",
        &format!("{base}/chat"),
        request.headers.get("authorization").map(String::as_str),
        Some(&request.body),
    )?;
    write_proxy_response(stream, response)
}

fn run_curl_json(
    method: &str,
    url: &str,
    authorization: Option<&str>,
    body: Option<&[u8]>,
) -> Result<ProxyResponse> {
    let mut command = Command::new("curl");
    command.args(["-sS", "-w", "\n%{http_code}", "-X", method, url]);
    command.args(["-H", "Accept: application/json"]);
    if body.is_some() {
        command.args(["-H", "Content-Type: application/json"]);
        command.args(["--data-binary", "@-"]);
        command.stdin(Stdio::piped());
    }
    if let Some(value) = authorization {
        command.arg("-H").arg(format!("Authorization: {value}"));
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn().context("start dashboard network bridge")?;
    if let Some(bytes) = body {
        let mut stdin = child.stdin.take().context("open dashboard bridge stdin")?;
        stdin
            .write_all(bytes)
            .context("write dashboard bridge request body")?;
    }
    let output = child
        .wait_with_output()
        .context("wait for dashboard network bridge")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("dashboard network bridge failed: {stderr}");
    }
    let stdout = String::from_utf8(output.stdout).context("decode dashboard bridge response")?;
    let Some((body, status_text)) = stdout.rsplit_once('\n') else {
        bail!("dashboard network bridge returned malformed response");
    };
    let status = status_text
        .trim()
        .parse::<u16>()
        .context("parse dashboard bridge status")?;
    Ok(ProxyResponse {
        status,
        body: body.as_bytes().to_vec(),
    })
}

fn write_proxy_response(stream: &mut impl Write, response: ProxyResponse) -> Result<()> {
    write_response(
        stream,
        response.status,
        "application/json; charset=utf-8",
        &response.body,
    )
}

fn ollama_base(request: &HttpRequest) -> Result<String> {
    let raw = request
        .headers
        .get("x-ollama-base")
        .map(String::as_str)
        .unwrap_or(OLLAMA_API_BASE);
    let trimmed = raw.trim().trim_end_matches('/');
    let normalized = if trimmed == "https://ollama.com" {
        OLLAMA_API_BASE.to_string()
    } else {
        trimmed.to_string()
    };
    if normalized != OLLAMA_API_BASE {
        bail!("unsupported Ollama API base: {raw}");
    }
    Ok(normalized)
}

fn request_path(raw: &str) -> PathBuf {
    let path = raw.split('?').next().unwrap_or("/");
    let path = percent_decode(path);
    if path == "/" {
        return PathBuf::from("index.html");
    }
    path.trim_start_matches('/')
        .split('/')
        .filter(|part| !part.is_empty() && *part != "." && *part != "..")
        .collect()
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                out.push(hex);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut impl Write,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        _ => "Proxy Response",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Length: {}\r\nContent-Type: {content_type}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )
    .context("write dashboard response headers")?;
    stream
        .write_all(body)
        .context("write dashboard response body")
}

trait ReadWrite: Read + Write {}

impl<T: Read + Write> ReadWrite for T {}

struct HttpRequest {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

struct ProxyResponse {
    status: u16,
    body: Vec<u8>,
}
