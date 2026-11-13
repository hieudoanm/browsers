use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use browsers_cli::serve::{serve_with_shutdown, ServeConfig};

const FIXTURE_HTML: &str = "<html><head><title>serve fixture</title></head>\
                           <body><h1>Hi from Servo</h1></body></html>";

#[test]
fn serve_end_to_end() {
    let fixture = fixture_server(FIXTURE_HTML);
    let (api, stop, serve_thread) = spawn_api();

    wait_until_healthy(&api);

    let (status, body) = http_get(&api, "/api/v1/health");
    assert_eq!(status, 200);
    assert!(body.contains("\"status\":\"ok\""), "body: {body}");

    let (status, body) = http_get(&api, "/api/v1/version");
    assert_eq!(status, 200);
    assert!(!body.trim().is_empty(), "empty version");

    let (status, headers, body_bytes) = http_post_full(
        &api,
        "/api/v1/scrape",
        &format!(r#"{{"url":"http://{fixture}/"}}"#),
    );
    let body = String::from_utf8_lossy(&body_bytes).into_owned();
    assert_eq!(status, 200, "body: {body}");
    assert!(
        body.contains("Hi from Servo"),
        "body: {}",
        &body[..200.min(body.len())]
    );
    let header = |name: &str| {
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(header("x-browserverless-load-status"), Some("ok"));
    let expected_url = format!("http://{fixture}/");
    assert_eq!(header("x-browserverless-url"), Some(expected_url.as_str()));
    assert_eq!(header("x-browserverless-title"), Some("serve fixture"));
    assert!(header("x-browserverless-memory-kb").is_some());
    assert!(header("x-browserverless-duration-ms").is_some());

    let (status, body) = http_post(&api, "/api/v1/scrape", r#"{"url":"file:///etc/passwd"}"#);
    assert_eq!(status, 400);
    assert!(body.contains("unsupported scheme"), "body: {body}");

    let (status, body) = http_post(&api, "/api/v1/scrape", "{}");
    assert_eq!(status, 400);
    assert!(body.contains("missing"), "body: {body}");

    let (status, body) = http_post(&api, "/api/v1/scrape", r#"{"url": 42}"#);
    assert_eq!(status, 400);
    assert!(body.contains("not a string"), "body: {body}");

    let (status, body) = http_get(&api, "/api/v1/scrape");
    assert_eq!(status, 405);
    assert!(body.contains("method not allowed"), "body: {body}");

    let (status, body) = http_get(&api, "/api/v1/nope");
    assert_eq!(status, 404);
    assert!(body.contains("not found"), "body: {body}");

    let fixture = fixture_server(FIXTURE_HTML);
    let (status, headers, body_bytes) = http_post_full(
        &api,
        "/api/v1/screenshot",
        &format!(r#"{{"url":"http://{fixture}/"}}"#),
    );
    assert_eq!(status, 200);
    assert!(
        body_bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "not a PNG: {} bytes",
        body_bytes.len()
    );
    assert!(headers
        .iter()
        .any(|(k, v)| { k.eq_ignore_ascii_case("content-type") && v == "image/png" }));
    let sh = |name: &str| {
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(sh("x-browserverless-load-status"), Some("ok"));
    let expected_url = format!("http://{fixture}/");
    assert_eq!(sh("x-browserverless-url"), Some(expected_url.as_str()));
    assert_eq!(sh("x-browserverless-title"), Some("serve fixture"));
    assert!(sh("x-browserverless-memory-kb").is_some());

    let (status, body) = http_post(
        &api,
        "/api/v1/screenshot",
        r#"{"url":"file:///etc/passwd"}"#,
    );
    assert_eq!(status, 400);
    assert!(body.contains("unsupported scheme"), "body: {body}");

    let (status, body) = http_get(&api, "/api/v1/screenshot");
    assert_eq!(status, 405);
    assert!(body.contains("method not allowed"), "body: {body}");

    // Stop the serve loop and join: dropping the HeadlessBrowser lets Servo
    // stop its engine threads cleanly, so process exit does not race with
    // live SpiderMonkey threads (which crashed CI with a SIGSEGV).
    stop.store(true, Ordering::Relaxed);
    serve_thread
        .join()
        .expect("serve thread panicked while shutting down");
}

fn spawn_api() -> (String, Arc<AtomicBool>, std::thread::JoinHandle<()>) {
    let server = tiny_http::Server::http("127.0.0.1:0").expect("bind api server");
    let addr = server.server_addr().to_ip().expect("ip addr").to_string();
    let config = ServeConfig {
        bind: "127.0.0.1:0".into(),
        port: None,
        viewport_width: 800,
        viewport_height: 600,
        load_timeout_ms: 15_000,
    };
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = stop.clone();

    let thread = std::thread::spawn(move || {
        let _ = serve_with_shutdown(config, server, &stop_thread);
    });

    (addr, stop, thread)
}

fn wait_until_healthy(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(60);
    while !probe_healthy(addr) {
        assert!(Instant::now() < deadline, "server did not become healthy");
        std::thread::sleep(Duration::from_millis(250));
    }
}

// Serves FIXTURE_HTML for exactly one HTTP request on an ephemeral port.
fn fixture_server(html: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let addr = listener.local_addr().expect("fixture addr").to_string();

    std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("fixture accept");
        let mut header = Vec::new();
        let mut byte = [0u8; 1];
        while !header.ends_with(b"\r\n\r\n") && stream.read(&mut byte).is_ok_and(|n| n > 0) {
            header.push(byte[0]);
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            html.len(),
            html
        );
        let _ = stream.write_all(response.as_bytes());
    });

    addr
}

fn http_get(addr: &str, path: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).expect("connect api");
    let request = format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).expect("write request");

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).expect("read response");
    parse_http(&buf)
}

fn http_post(addr: &str, path: &str, body: &str) -> (u16, String) {
    let (status, _, body) = http_post_full(addr, path, body);
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn http_post_full(addr: &str, path: &str, body: &str) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let mut stream = TcpStream::connect(addr).expect("connect api");
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(request.as_bytes()).expect("write request");

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).expect("read response");
    parse_http_full(&buf)
}

// Tolerant probe: the server may not be listening yet during startup.
fn probe_healthy(addr: &str) -> bool {
    let Ok(mut stream) = TcpStream::connect(addr) else {
        return false;
    };
    let request =
        format!("GET /api/v1/health HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n");
    let _ = stream.write_all(request.as_bytes());
    let mut buf = Vec::new();
    if stream.read_to_end(&mut buf).is_err() {
        return false;
    }
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .is_some_and(|_| parse_http(&buf).0 == 200)
}

fn parse_http(buf: &[u8]) -> (u16, String) {
    let (status, _, body) = parse_http_full(buf);
    (status, String::from_utf8_lossy(&body).into_owned())
}

fn parse_http_full(buf: &[u8]) -> (u16, Vec<(String, String)>, Vec<u8>) {
    let head_end = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .expect("header end")
        + 4;
    let head = std::str::from_utf8(&buf[..head_end]).expect("head utf8");
    let mut lines = head.split("\r\n");
    let status_line = lines.next().expect("status line");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .expect("status code")
        .parse()
        .expect("parse status");
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    let body = buf[head_end..].to_vec();
    (status, headers, body)
}
