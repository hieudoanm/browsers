//! Real-render regression test for the Servo-backed MCP renderer.
//!
//! Servo initialises process-wide state that panics when a second browser is
//! constructed, so this file deliberately contains a single test: two tests
//! would build two browsers in one process and panic. `serve_api.rs` is a
//! separate test binary, and so a separate process, so it may own its own
//! browser.

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

use browsers_cli::mcp::handlers::{Renderer, ServeRenderer};

const FIXTURE_HTML: &str = "<html><head><title>mcp fixture</title></head>\
                           <body><h1>Hello MCP</h1></body></html>";

/// fixture_server serves body on an ephemeral port after an optional delay,
/// then returns its authority, so the test never touches the public internet.
fn fixture_server(body: &'static str, delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    let authority = listener.local_addr().expect("fixture address").to_string();
    thread::spawn(move || {
        for stream in listener.incoming() {
            // Handle each connection on its own thread so a deliberately slow
            // fixture cannot block the accept loop.
            thread::spawn(move || {
                let Ok(mut stream) = stream else { return };
                if !delay.is_zero() {
                    thread::sleep(delay);
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            });
        }
    });
    authority
}

/// wait_for_http pings the fixture until it accepts a connection, so the test
/// never races the server thread. It deliberately does not read the response:
/// a deliberately slow fixture would block this probe for its full delay.
fn wait_for_http(authority: &str) {
    for _ in 0..100 {
        if let Ok(mut stream) = TcpStream::connect(authority) {
            let _ = stream.write_all(b"HEAD / HTTP/1.0\r\n\r\n");
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }
    panic!("fixture server at {authority} never became ready");
}

#[test]
fn one_renderer_serves_several_calls_without_reinitialising_servo() {
    let fast = fixture_server(FIXTURE_HTML, Duration::ZERO);
    let slow = fixture_server(FIXTURE_HTML, Duration::from_secs(30));
    wait_for_http(&fast);
    wait_for_http(&slow);

    // Servo owns one event loop per process, so a session builds a single
    // browser on first use and gives every call a fresh page.
    let renderer = ServeRenderer::new(800, 600, 60_000);

    let first = renderer
        .scrape(&format!("http://{fast}/"), Some(60_000))
        .expect("first scrape must render");
    assert!(
        first.html.contains("Hello MCP"),
        "first scrape html: url={} timed_out={} html={:.400}",
        first.url,
        first.timed_out,
        first.html
    );
    assert_eq!(first.title, "mcp fixture");
    assert!(!first.timed_out, "the fixture responds immediately");

    let shot = renderer
        .screenshot(&format!("http://{fast}/"), Some(60_000))
        .expect("screenshot must render after a scrape");
    assert_eq!(&shot.png[..4], b"\x89PNG", "screenshot must be a PNG");
    assert!(
        shot.png.len() > 1_000,
        "a blank PNG would be suspiciously small"
    );

    // The regression: before the browser was cached, this second scrape built a
    // second browser and Servo panicked with "Already initialized".
    let second = renderer
        .scrape(&format!("http://{fast}/"), Some(60_000))
        .expect("second scrape must render after a screenshot");
    assert!(
        second.html.contains("Hello MCP"),
        "second scrape html: url={} timed_out={} html={:.400}",
        second.url,
        second.timed_out,
        second.html
    );

    // A per-call deadline shorter than the server's 30s response must bound the
    // call. Servo reports a page that never sends headers as a load failure
    // rather than yielding partial DOM, so the contract is a reported error in
    // roughly the requested time, not a hang and not a panic.
    let started = Instant::now();
    let error = renderer
        .scrape(&format!("http://{slow}/"), Some(2_000))
        .expect_err("a page that never responds must be reported as an error");
    let elapsed = started.elapsed();
    assert!(
        elapsed < Duration::from_secs(20),
        "a 2s deadline must not wait for a 30s response, took {elapsed:?}"
    );
    assert!(!error.is_empty(), "an error must explain itself");
}
