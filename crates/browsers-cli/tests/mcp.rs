//! End-to-end tests for the browserverless MCP server.
//!
//! Rendering is stubbed so the protocol contract can be tested without booting
//! Servo; the real renderer is covered by the `headless` crate's own tests.

use std::io::Cursor;

use browsers_cli::mcp::handlers::{register, Renderer, ScrapeOutcome, ScreenshotOutcome};
use browsers_cli::mcp::tools::{scrape_tool, screenshot_tool, version_tool};
use browsers_cli::mcp::Server;
use serde_json::{json, Value};

/// StubRenderer returns canned output and records the URLs it was asked for.
struct StubRenderer {
    calls: std::sync::Mutex<Vec<String>>,
    scrape_error: Option<String>,
}

impl StubRenderer {
    fn new() -> Self {
        StubRenderer {
            calls: std::sync::Mutex::new(Vec::new()),
            scrape_error: None,
        }
    }

    fn failing(message: &str) -> Self {
        StubRenderer {
            calls: std::sync::Mutex::new(Vec::new()),
            scrape_error: Some(message.to_string()),
        }
    }
}

impl Renderer for StubRenderer {
    fn scrape(&self, url: &str, _timeout_ms: Option<u64>) -> Result<ScrapeOutcome, String> {
        self.calls.lock().expect("calls").push(url.to_string());
        if let Some(message) = &self.scrape_error {
            return Err(message.clone());
        }
        Ok(ScrapeOutcome {
            url: "https://example.com/final".to_string(),
            title: "Example".to_string(),
            html: "<html><body>hi</body></html>".to_string(),
            timed_out: false,
            duration_ms: 12,
            memory_kb: 34,
        })
    }

    fn screenshot(&self, url: &str, _timeout_ms: Option<u64>) -> Result<ScreenshotOutcome, String> {
        self.calls.lock().expect("calls").push(url.to_string());
        Ok(ScreenshotOutcome {
            url: "https://example.com/final".to_string(),
            title: "Example".to_string(),
            png: vec![0x89, b'P', b'N', b'G'],
            timed_out: true,
            duration_ms: 20,
            memory_kb: 56,
        })
    }
}

/// drive runs the given input through a server with the stub renderer, returning
/// the response lines.
fn drive(renderer: StubRenderer, input: &str) -> Vec<Value> {
    let mut out: Vec<u8> = Vec::new();
    let mut server = Server::new(&mut out);
    for (tool, handler) in register(renderer) {
        server.add_tool(tool, handler);
    }
    server
        .serve(Cursor::new(input.as_bytes().to_vec()))
        .expect("serve");
    drop(server);
    String::from_utf8(out)
        .expect("utf-8 stdout")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("each line is a JSON frame"))
        .collect()
}

/// call builds a tools/call frame for a tool.
fn call(id: u32, name: &str, arguments: Value) -> String {
    json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": name, "arguments": arguments}})
        .to_string()
        + "\n"
}

#[test]
fn initialize_reports_the_protocol_version_and_identity() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}})
        ),
    );
    let result = &responses[0]["result"];
    assert_eq!(result["protocolVersion"], "2025-11-25");
    assert_eq!(result["serverInfo"]["name"], "browserverless-mcp");
    assert_eq!(result["capabilities"]["tools"]["listChanged"], false);
}

#[test]
fn an_unsupported_protocol_version_falls_back_to_the_served_one() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"1999-01-01"}})
        ),
    );
    assert_eq!(responses[0]["result"]["protocolVersion"], "2025-11-25");
}

#[test]
fn ping_is_answered_with_an_empty_result() {
    let responses = drive(
        StubRenderer::new(),
        &format!("{}\n", json!({"jsonrpc":"2.0","id":1,"method":"ping"})),
    );
    assert_eq!(responses[0]["result"], json!({}));
}

#[test]
fn the_tool_list_is_sorted_and_complete() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})
        ),
    );
    let names: Vec<&str> = responses[0]["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().expect("name"))
        .collect();
    assert_eq!(
        names,
        vec![
            "browserverless_scrape",
            "browserverless_screenshot",
            "browserverless_version"
        ]
    );
}

#[test]
fn scrape_returns_html_as_text() {
    let renderer = StubRenderer::new();
    let responses = drive(
        renderer,
        &call(
            1,
            "browserverless_scrape",
            json!({"url": "https://example.com"}),
        ),
    );

    let result = &responses[0]["result"];
    assert!(
        result.get("isError").is_none(),
        "a success must omit isError"
    );
    let payload: Value = serde_json::from_str(result["content"][0]["text"].as_str().expect("text"))
        .expect("payload json");
    assert_eq!(payload["html"], "<html><body>hi</body></html>");
    assert_eq!(payload["title"], "Example");
    assert_eq!(payload["memory_kb"], 34);
}

#[test]
fn screenshot_returns_the_png_as_an_image_block_not_as_text() {
    let responses = drive(
        StubRenderer::new(),
        &call(
            1,
            "browserverless_screenshot",
            json!({"url": "https://example.com"}),
        ),
    );

    let content = responses[0]["result"]["content"]
        .as_array()
        .expect("content");
    assert_eq!(content.len(), 2, "a summary and an image");

    let summary: Value =
        serde_json::from_str(content[0]["text"].as_str().expect("text")).expect("summary json");
    assert_eq!(summary["png_bytes"], 4);
    assert!(
        content[0].get("data").is_none(),
        "the summary must not inline image bytes"
    );

    assert_eq!(content[1]["type"], "image");
    assert_eq!(content[1]["mimeType"], "image/png");
    // Standard base64 of 89 50 4E 47.
    assert_eq!(content[1]["data"], "iVBORw==");
}

#[test]
fn a_render_failure_is_a_tool_error_not_a_jsonrpc_error() {
    let responses = drive(
        StubRenderer::failing("connection refused"),
        &call(
            1,
            "browserverless_scrape",
            json!({"url": "https://example.com"}),
        ),
    );

    assert!(
        responses[0].get("error").is_none(),
        "a tool failure must not become a JSON-RPC error"
    );
    assert_eq!(responses[0]["result"]["isError"], true);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("connection refused"), "{text}");
}

#[test]
fn a_missing_url_is_reported_to_the_model_in_its_own_terms() {
    let responses = drive(
        StubRenderer::new(),
        &call(1, "browserverless_scrape", json!({})),
    );
    assert_eq!(responses[0]["result"]["isError"], true);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert_eq!(text, "missing required argument: url");
}

#[test]
fn a_non_http_url_is_refused() {
    let responses = drive(
        StubRenderer::new(),
        &call(
            1,
            "browserverless_scrape",
            json!({"url": "file:///etc/passwd"}),
        ),
    );
    assert_eq!(responses[0]["result"]["isError"], true);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("unsupported scheme"), "{text}");
}

#[test]
fn a_oversized_timeout_is_refused_rather_than_disabling_the_deadline() {
    let responses = drive(
        StubRenderer::new(),
        &call(
            1,
            "browserverless_scrape",
            json!({"url": "https://example.com", "timeout_ms": 9223372036854775807i64}),
        ),
    );
    assert_eq!(responses[0]["result"]["isError"], true);
    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(text.contains("must not exceed"), "{text}");
}

#[test]
fn an_unknown_tool_is_a_method_not_found_error() {
    let responses = drive(StubRenderer::new(), &call(1, "browsers_nope", json!({})));
    assert_eq!(responses[0]["error"]["code"], -32601);
    assert!(responses[0]["error"]["message"]
        .as_str()
        .expect("message")
        .contains("tool not found"));
}

#[test]
fn an_unknown_method_is_a_method_not_found_error() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"resources/list"})
        ),
    );
    assert_eq!(responses[0]["error"]["code"], -32601);
}

#[test]
fn a_malformed_frame_is_a_parse_error_with_a_null_id() {
    let responses = drive(StubRenderer::new(), "{not json}\n");
    assert_eq!(responses[0]["error"]["code"], -32700);
    assert_eq!(responses[0]["id"], Value::Null);
}

#[test]
fn a_wrong_jsonrpc_version_is_an_invalid_request() {
    let responses = drive(
        StubRenderer::new(),
        &format!("{}\n", json!({"jsonrpc":"1.0","id":1,"method":"ping"})),
    );
    assert_eq!(responses[0]["error"]["code"], -32600);
}

#[test]
fn a_notification_is_never_answered() {
    let note = json!({"jsonrpc":"2.0","method":"notifications/initialized"}).to_string();
    let input = format!("{note}\n{}", call(2, "browserverless_version", json!({})));
    let responses = drive(StubRenderer::new(), &input);

    assert_eq!(
        responses.len(),
        1,
        "only the id-bearing request may be answered"
    );
    assert_eq!(responses[0]["id"], 2);
}

#[test]
fn a_notification_with_a_bad_version_stays_silent() {
    // The notification check must precede the version check, or the client
    // receives a reply it can never match.
    let input = format!("{}\n", json!({"jsonrpc":"1.0","method":"ping"}));
    assert!(drive(StubRenderer::new(), &input).is_empty());
}

#[test]
fn a_notification_does_not_run_its_tool() {
    let renderer = StubRenderer::new();
    let input = format!(
        "{}\n",
        json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"browserverless_scrape","arguments":{"url":"https://example.com"}}})
    );
    let responses = drive(renderer, &input);

    assert!(responses.is_empty(), "a notification must produce no reply");
}

#[test]
fn a_call_with_no_params_names_no_tool_rather_than_failing_to_decode() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call"})
        ),
    );
    assert_eq!(responses[0]["error"]["code"], -32601);
    assert!(responses[0]["error"]["message"]
        .as_str()
        .expect("message")
        .contains("tool not found"));
}

#[test]
fn a_call_with_null_params_names_no_tool() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":null})
        ),
    );
    assert_eq!(responses[0]["error"]["code"], -32601);
}

#[test]
fn non_object_params_are_rejected_as_invalid_params() {
    let responses = drive(
        StubRenderer::new(),
        &format!(
            "{}\n",
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":"nope"})
        ),
    );
    assert_eq!(responses[0]["error"]["code"], -32602);
}

#[test]
fn every_response_is_a_single_line() {
    // A pretty-printed reply would split across lines and desynchronise a
    // line-oriented client, so the transport is exercised over multi-line input.
    let list = json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}).to_string();
    let input = format!(
        "{list}\n{}",
        call(
            2,
            "browserverless_scrape",
            json!({"url":"https://example.com"})
        )
    );
    let mut out: Vec<u8> = Vec::new();
    let mut server = Server::new(&mut out);
    for (tool, handler) in register(StubRenderer::new()) {
        server.add_tool(tool, handler);
    }
    server
        .serve(Cursor::new(input.into_bytes()))
        .expect("serve");
    drop(server);
    let text = String::from_utf8(out).expect("utf-8");
    let lines: Vec<&str> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        serde_json::from_str::<Value>(line).expect("each line parses as one frame");
    }
}

#[test]
fn the_version_tool_reports_the_binary_version() {
    let responses = drive(
        StubRenderer::new(),
        &call(1, "browserverless_version", json!({})),
    );
    let payload: Value = serde_json::from_str(
        responses[0]["result"]["content"][0]["text"]
            .as_str()
            .expect("text"),
    )
    .expect("payload");
    assert_eq!(payload["server"], "browserverless-mcp");
    assert!(!payload["version"].as_str().expect("version").is_empty());
}

#[test]
fn the_tool_schemas_declare_url_as_required() {
    for tool in [scrape_tool(), screenshot_tool()] {
        assert_eq!(tool.input_schema["type"], "object");
        assert_eq!(tool.input_schema["required"], json!(["url"]));
        assert_eq!(tool.input_schema["properties"]["url"]["type"], "string");
    }
    assert_eq!(version_tool().input_schema["required"], Value::Null);
}

#[test]
fn the_stub_renderer_receives_the_validated_url() {
    let renderer = StubRenderer::new();
    let _ = drive(
        renderer,
        &call(
            1,
            "browserverless_scrape",
            json!({"url": "https://example.com/page"}),
        ),
    );
}
