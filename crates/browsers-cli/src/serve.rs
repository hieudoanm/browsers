use std::io::Read;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use browsers_headless::{HeadlessBrowser, HeadlessConfig, HeadlessError};
use tiny_http::{Header, Method, Request, Response, Server};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_BODY_BYTES: u64 = 64 * 1024;
const DEFAULT_PORT: u16 = 8080;

static REQUEST_ID: AtomicU64 = AtomicU64::new(0);

pub struct ServeConfig {
    pub bind: String,
    pub port: Option<u16>,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub load_timeout_ms: u64,
}

fn resolve_bind(bind: &str, port: Option<u16>) -> String {
    match (bind.rsplit_once(':'), port) {
        (None, None) => format!("{bind}:{DEFAULT_PORT}"),
        (None, Some(p)) => format!("{bind}:{p}"),
        (Some(_), None) => bind.to_string(),
        (Some((host, _)), Some(p)) => format!("{host}:{p}"),
    }
}

pub fn run(config: ServeConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind = resolve_bind(&config.bind, config.port);
    let server = Server::http(bind.as_str())?;

    let addr = server.server_addr().to_ip();
    let port = addr.map(|a| a.port()).unwrap_or(DEFAULT_PORT);
    let ip = external_ipv4().unwrap_or_else(|| {
        addr.map(|a| a.ip().to_string())
            .unwrap_or_else(|| "127.0.0.1".into())
    });
    print_listening_banner(&ip, port);

    serve(config, server)
}

fn external_ipv4() -> Option<String> {
    let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();
    if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
        return None;
    }
    let virtual_ifaces = [
        "lo", "utun", "awdl", "llw", "bridge", "vmnet", "vbox", "tap", "tun",
    ];
    let mut candidate: Option<String> = None;
    let mut cursor = ifaddrs;
    unsafe {
        while !cursor.is_null() {
            let entry = &*cursor;
            let name = std::ffi::CStr::from_ptr(entry.ifa_name).to_string_lossy();
            let is_virtual = virtual_ifaces.iter().any(|p| name.starts_with(p));
            let family = if entry.ifa_addr.is_null() {
                0
            } else {
                (*entry.ifa_addr).sa_family as i32
            };
            if family == libc::AF_INET && !is_virtual {
                let sin = entry.ifa_addr as *const libc::sockaddr_in;
                let ipv4 = std::net::Ipv4Addr::from(u32::from_be((*sin).sin_addr.s_addr));
                if !ipv4.is_loopback() && !ipv4.is_link_local() && candidate.is_none() {
                    candidate = Some(ipv4.to_string());
                }
            }
            cursor = entry.ifa_next;
        }
        libc::freeifaddrs(ifaddrs);
    }
    candidate
}

static USE_COLOR: OnceLock<bool> = OnceLock::new();

fn use_color() -> bool {
    *USE_COLOR.get_or_init(|| unsafe { libc::isatty(libc::STDOUT_FILENO) == 1 })
}

fn paint(style: &str, text: &str) -> String {
    if use_color() {
        format!("\x1b[{style}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

fn status_ansi(status: u16) -> &'static str {
    match status {
        200..=299 => "32",
        300..=399 => "36",
        400..=499 => "33",
        _ => "31",
    }
}

fn paint_status(status: u16) -> String {
    paint(status_ansi(status), &status.to_string())
}

fn print_listening_banner(ip: &str, port: u16) {
    let title = "Browserverless server";
    let rows = [
        ("localhost", format!("http://127.0.0.1:{port}")),
        ("ip:port", format!("http://{ip}:{port}")),
    ];
    let width = [title.len()]
        .into_iter()
        .chain(rows.iter().map(|(_, v)| 9 + 2 + v.len()))
        .max()
        .unwrap_or(0);
    let horizontal = "─".repeat(width + 2);
    println!("╭{horizontal}╮");
    println!(
        "│ {}{} │",
        paint("1;36", title),
        " ".repeat(width - title.len())
    );
    for (label, value) in &rows {
        println!(
            "│ {}{}{} │",
            paint("2", &format!("{label:<9}  ")),
            paint("32", value),
            " ".repeat(width - (9 + 2 + value.len()))
        );
    }
    println!("╰{horizontal}╯");
}

pub fn serve(
    config: ServeConfig,
    server: Server,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    serve_with_shutdown(config, server, &AtomicBool::new(false))
}

/// Like [`serve`], but returns as soon as `stop` is set (checked between
/// requests) or the server is closed. Returning drops the `HeadlessBrowser`,
/// which lets Servo shut down its engine threads cleanly before the process
/// exits.
pub fn serve_with_shutdown(
    config: ServeConfig,
    server: Server,
    stop: &AtomicBool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let browser = HeadlessBrowser::new(HeadlessConfig {
        viewport_width: config.viewport_width,
        viewport_height: config.viewport_height,
        load_timeout_ms: config.load_timeout_ms,
        wait_after_load_ms: 500,
    })?;

    loop {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match server.recv_timeout(Duration::from_millis(100)) {
            Ok(Some(mut request)) => {
                let request_id = REQUEST_ID.fetch_add(1, Ordering::Relaxed) + 1;
                let method = request.method().to_string();
                let path = request.url().to_string();
                let started = Instant::now();
                let (status, response) = route(&browser, &mut request);
                let _ = request.respond(response);
                let duration_ms = started.elapsed().as_millis();
                println!(
                    "{} {} {} {path} {} {} {}",
                    paint("2", &format!("[{}]", now_str())),
                    paint("2", &format!("req={request_id}")),
                    paint("36", &method),
                    paint("2", "->"),
                    paint_status(status),
                    paint("2", &format!("({duration_ms}ms)"))
                );
            }
            Ok(None) => continue,
            // Server unblocked/closed (e.g. a graceful-shutdown signal).
            Err(_) => break,
        }
    }
    Ok(())
}

fn now_str() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as libc::time_t)
        .unwrap_or(0);
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe {
        libc::localtime_r(&secs, &mut tm);
    }
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        tm.tm_year + 1900,
        tm.tm_mon + 1,
        tm.tm_mday,
        tm.tm_hour,
        tm.tm_min,
        tm.tm_sec
    )
}

#[derive(Debug, thiserror::Error)]
pub enum ScrapeError {
    #[error("missing '{0}' in json body")]
    MissingField(String),

    #[error("invalid json body: {0}")]
    InvalidJson(String),
}

fn route(
    browser: &HeadlessBrowser,
    request: &mut Request,
) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let path = request.url().split('?').next().unwrap_or("");

    match (request.method(), path) {
        (&Method::Get, "/api/v1/health") => json(200, r#"{"status":"ok"}"#.to_string()),
        (&Method::Get, "/api/v1/version") => text(200, VERSION.to_string()),
        (&Method::Get, "/api/v1/openapi.json") => json(200, openapi_spec()),
        (&Method::Get, "/docs") => openapi_docs_page(),
        (&Method::Post, "/api/v1/scrape") => {
            let url = read_body(request)
                .map_err(ScrapeError::InvalidJson)
                .and_then(|b| extract_json_string(&b, "url"));
            scrape_target(browser, url)
        }
        (&Method::Post, "/api/v1/screenshot") => {
            let url = read_body(request)
                .map_err(ScrapeError::InvalidJson)
                .and_then(|b| extract_json_string(&b, "url"));
            screenshot_target(browser, url)
        }
        (method, "/api/v1/scrape" | "/api/v1/screenshot") => {
            debug_assert_ne!(method, &Method::Post);
            json(405, error_body("method not allowed"))
        }
        _ => json(404, error_body("not found")),
    }
}

fn scrape_target(
    browser: &HeadlessBrowser,
    url: Result<String, ScrapeError>,
) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let raw_url = match url {
        Ok(u) => u,
        Err(e) => return json(400, error_body(&e.to_string())),
    };

    let target = match validate_target_url(&raw_url) {
        Ok(t) => t,
        Err(e) => return json(400, error_body(&e)),
    };

    match browser.scrape(&target) {
        Ok(result) => {
            let headers = meta_headers(
                &result.url,
                &result.title,
                result.timed_out,
                result.memory_kb,
                result.duration_ms,
            );
            text_with_headers(200, result.html, &headers)
        }
        Err(HeadlessError::Timeout(_)) => json(504, error_body("render timed out")),
        Err(e) => json(500, error_body(&format!("render failed: {e}"))),
    }
}

fn screenshot_target(
    browser: &HeadlessBrowser,
    url: Result<String, ScrapeError>,
) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let raw_url = match url {
        Ok(u) => u,
        Err(e) => return json(400, error_body(&e.to_string())),
    };

    let target = match validate_target_url(&raw_url) {
        Ok(t) => t,
        Err(e) => return json(400, error_body(&e)),
    };

    match browser.screenshot_bytes(&target) {
        Ok(result) => {
            let headers = meta_headers(
                &result.url,
                &result.title,
                result.timed_out,
                result.memory_kb,
                result.duration_ms,
            );
            png(200, result.png, &headers)
        }
        Err(HeadlessError::Timeout(_)) => json(504, error_body("render timed out")),
        Err(e) => json(500, error_body(&format!("screenshot failed: {e}"))),
    }
}

fn meta_headers(
    url: &str,
    title: &str,
    timed_out: bool,
    memory_kb: u64,
    duration_ms: u128,
) -> Vec<(&'static str, String)> {
    vec![
        ("x-browserverless-url", header_safe(url)),
        ("x-browserverless-title", header_safe(title)),
        (
            "x-browserverless-load-status",
            if timed_out { "partial" } else { "ok" }.to_string(),
        ),
        ("x-browserverless-memory-kb", memory_kb.to_string()),
        ("x-browserverless-duration-ms", duration_ms.to_string()),
    ]
}

// Response headers must be ASCII; strip non-printable/control characters to
// prevent header injection from untrusted page titles or URLs.
fn header_safe(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_graphic() || c == ' ' {
                c
            } else {
                '?'
            }
        })
        .collect()
}

fn validate_target_url(raw: &str) -> Result<String, String> {
    let parsed = url::Url::parse(raw).map_err(|e| format!("invalid url: {e}"))?;

    match parsed.scheme() {
        "http" | "https" => Ok(parsed.to_string()),
        other => Err(format!("unsupported scheme: {other}")),
    }
}

fn read_body(request: &mut Request) -> Result<String, String> {
    let mut buf = String::new();
    request
        .as_reader()
        .take(MAX_BODY_BYTES)
        .read_to_string(&mut buf)
        .map_err(|e| format!("failed to read body: {e}"))?;
    Ok(buf)
}

// Minimal JSON string-field extractor: finds `"field": "value"` and decodes
// the value with standard string escapes (surrogate pairs unsupported).
fn extract_json_string(body: &str, field: &str) -> Result<String, ScrapeError> {
    let token = format!("\"{field}\"");
    let start = body
        .find(&token)
        .ok_or_else(|| ScrapeError::MissingField(field.to_string()))?;
    let rest = body[start + token.len()..].trim_start();
    let rest = rest
        .strip_prefix(':')
        .ok_or_else(|| ScrapeError::InvalidJson("expected ':'".into()))?
        .trim_start();
    let rest = rest
        .strip_prefix('"')
        .ok_or_else(|| ScrapeError::InvalidJson(format!("'{field}' is not a string")))?;

    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Ok(out),
            '\\' => match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    let decoded = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32);
                    if let Some(cp) = decoded {
                        out.push(cp);
                    }
                }
                Some(other) => out.push(other),
                None => return Err(ScrapeError::InvalidJson("trailing backslash".into())),
            },
            c => out.push(c),
        }
    }
    Err(ScrapeError::InvalidJson("unterminated string".into()))
}

fn openapi_spec() -> String {
    include_str!("../../../api/openapi.json").replace("0.0.0", VERSION)
}

fn openapi_docs_page() -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let body = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Browserverless API Docs</title>
  <link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700">
</head>
<body>
  <redoc spec-url="/api/v1/openapi.json"></redoc>
  <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"></script>
</body>
</html>"#;
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .expect("static header");
    let response = Response::from_string(body)
        .with_status_code(200)
        .with_header(header);
    (200, response)
}

fn json(status: u16, body: String) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let header = Header::from_bytes(
        &b"Content-Type"[..],
        &b"application/json; charset=utf-8"[..],
    )
    .expect("static header");
    let response = Response::from_string(body)
        .with_status_code(status)
        .with_header(header);
    (status, response)
}

fn text(status: u16, body: String) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    text_with_headers(status, body, &[])
}

fn text_with_headers(
    status: u16,
    body: String,
    headers: &[(&str, String)],
) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let content_type = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .expect("static header");
    let mut response = Response::from_string(body)
        .with_status_code(status)
        .with_header(content_type);
    for (name, value) in headers {
        let header = Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("header");
        response = response.with_header(header);
    }
    (status, response)
}

fn png(
    status: u16,
    data: Vec<u8>,
    headers: &[(&str, String)],
) -> (u16, Response<std::io::Cursor<Vec<u8>>>) {
    let content_type =
        Header::from_bytes(&b"Content-Type"[..], &b"image/png"[..]).expect("static header");
    let mut response = Response::from_data(data)
        .with_status_code(status)
        .with_header(content_type);
    for (name, value) in headers {
        let header = Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("header");
        response = response.with_header(header);
    }
    (status, response)
}

fn error_body(message: &str) -> String {
    let escaped = message.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"error\":\"{escaped}\"}}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_string_reads_field() {
        assert_eq!(
            extract_json_string(r#"{"url":"https://example.com/"}"#, "url").unwrap(),
            "https://example.com/"
        );
        assert_eq!(
            extract_json_string(r#"{ "url" : "https://example.com/" }"#, "url").unwrap(),
            "https://example.com/"
        );
        assert_eq!(
            extract_json_string(r#"{"other":1,"url":"http://a/b"}"#, "url").unwrap(),
            "http://a/b"
        );
    }

    #[test]
    fn extract_json_string_decodes_escapes() {
        assert_eq!(
            extract_json_string(r#"{"url":"a\nb \"x\" \\ категор"}"#, "url").unwrap(),
            "a\nb \"x\" \\ категор"
        );
        assert_eq!(
            extract_json_string(r#"{"url":"http://a/\u002fb"}"#, "url").unwrap(),
            "http://a//b"
        );
    }

    #[test]
    fn extract_json_string_rejects_malformed() {
        let err = extract_json_string(r#"{"other":1}"#, "url")
            .unwrap_err()
            .to_string();
        assert!(err.contains("missing 'url'"), "{err}");
        let err = extract_json_string(r#"{"url": 42}"#, "url")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a string"), "{err}");
        let err = extract_json_string(r#"{"url": try}"#, "url")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not a string"), "{err}");
        assert!(extract_json_string(r#"{"url""#, "url").is_err());
    }

    #[test]
    fn status_ansi_maps_by_class() {
        assert_eq!(status_ansi(200), "32");
        assert_eq!(status_ansi(204), "32");
        assert_eq!(status_ansi(301), "36");
        assert_eq!(status_ansi(404), "33");
        assert_eq!(status_ansi(504), "31");
    }

    #[test]
    fn resolve_bind_combines_host_and_port() {
        assert_eq!(resolve_bind("127.0.0.1:8080", None), "127.0.0.1:8080");
        assert_eq!(resolve_bind("127.0.0.1", None), "127.0.0.1:8080");
        assert_eq!(resolve_bind("127.0.0.1", Some(9000)), "127.0.0.1:9000");
        assert_eq!(resolve_bind("127.0.0.1:8080", Some(9001)), "127.0.0.1:9001");
        assert_eq!(resolve_bind("localhost", Some(7000)), "localhost:7000");
    }

    #[test]
    fn external_ipv4_is_not_loopback() {
        if let Some(ip) = external_ipv4() {
            assert!(!ip.starts_with("127."), "unexpected loopback {ip}");
        }
    }

    #[test]
    fn validate_accepts_http_and_https() {
        assert!(validate_target_url("https://example.com/a?b=c").is_ok());
        assert!(validate_target_url("http://127.0.0.1:8080/").is_ok());
    }

    #[test]
    fn validate_rejects_other_schemes() {
        let err = validate_target_url("file:///etc/passwd").unwrap_err();
        assert!(err.contains("unsupported scheme"), "{err}");
        let err = validate_target_url("data:text/plain,hi").unwrap_err();
        assert!(err.contains("unsupported scheme"), "{err}");
        let err = validate_target_url("not a valid url").unwrap_err();
        assert!(err.contains("invalid url"), "{err}");
    }

    #[test]
    fn header_safe_strips_control_and_non_ascii() {
        assert_eq!(header_safe("café\u{1F600}"), "caf??");
        assert_eq!(header_safe("a\nb\r\nc"), "a?b??c");
        assert_eq!(
            header_safe("https://example.com/a?b=c"),
            "https://example.com/a?b=c"
        );
    }

    #[test]
    fn error_body_escapes_quotes() {
        let escaped = error_body("bad \"url\"");
        assert!(escaped.contains(r#""error":"bad \"url\"""#), "{escaped}");
    }
}
