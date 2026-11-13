//! Tool handlers, each delegating rendering to the `headless` crate.
//!
//! Rendering is behind a trait so tests can exercise argument validation and
//! result shaping without booting Servo, and so the tools never depend on which
//! backend fulfils a call.

use std::sync::Mutex;

use base64::Engine as _;
use serde::Deserialize;
use serde_json::{json, Value};

use super::protocol::{ContentItem, ToolResult};
use super::tools::{scrape_tool, screenshot_tool, version_tool, Tool};
use super::ToolHandler;

/// MAX_TIMEOUT_MS caps a caller-supplied timeout. Values beyond it are rejected
/// rather than allowed to overflow the duration arithmetic, which would wrap to
/// a non-positive value and silently disable the deadline.
const MAX_TIMEOUT_MS: u64 = 10 * 60 * 1000;

/// Renderer produces page content. `browsers_headless::HeadlessBrowser` is the real
/// implementation; tests supply a stub.
///
/// The trait deliberately does not require `Send` or `Sync`: Servo's rendering
/// context is thread-affine, and the stdio server handles one request at a time.
pub trait Renderer {
    /// scrape renders a URL and returns its outcome.
    fn scrape(&self, url: &str, timeout_ms: Option<u64>) -> Result<ScrapeOutcome, String>;
    /// screenshot renders a URL and returns a PNG.
    fn screenshot(&self, url: &str, timeout_ms: Option<u64>) -> Result<ScreenshotOutcome, String>;
}

/// ScrapeOutcome is the text payload of the scrape tool.
#[derive(Debug, Clone)]
pub struct ScrapeOutcome {
    pub url: String,
    pub title: String,
    pub html: String,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub memory_kb: u64,
}

/// ScreenshotOutcome carries the PNG separately from the text summary.
#[derive(Debug, Clone)]
pub struct ScreenshotOutcome {
    pub url: String,
    pub title: String,
    pub png: Vec<u8>,
    pub timed_out: bool,
    pub duration_ms: u128,
    pub memory_kb: u64,
}

/// register returns the browserverless tool surface bound to a renderer.
pub fn register(renderer: impl Renderer + 'static) -> Vec<(Tool, ToolHandler)> {
    let shared = std::sync::Arc::new(renderer);
    vec![
        (scrape_tool(), handle_scrape(shared.clone())),
        (screenshot_tool(), handle_screenshot(shared)),
        (version_tool(), handle_version()),
    ]
}

/// ServeRenderer renders in-process through the Servo-backed headless crate.
pub struct ServeRenderer {
    config: Mutex<browsers_headless::HeadlessConfig>,
    /// Servo initialises process-wide state exactly once, so constructing a
    /// second browser panics. The browser is therefore built on first use and
    /// reused, giving every tool call a fresh page but a single event loop.
    browser: Mutex<Option<browsers_headless::HeadlessBrowser>>,
}

impl ServeRenderer {
    /// new records the viewport and default load timeout for this session.
    pub fn new(width: u32, height: u32, timeout_ms: u64) -> Self {
        ServeRenderer {
            config: Mutex::new(browsers_headless::HeadlessConfig {
                viewport_width: width,
                viewport_height: height,
                load_timeout_ms: timeout_ms,
                wait_after_load_ms: 0,
            }),
            browser: Mutex::new(None),
        }
    }

    /// with_browser runs a render against the session browser, creating it on
    /// first use. Holding the lock for the whole render keeps the thread-affine
    /// Servo context owned by exactly one caller.
    fn with_browser<T>(
        &self,
        render: impl FnOnce(
            &browsers_headless::HeadlessBrowser,
        ) -> Result<T, browsers_headless::HeadlessError>,
    ) -> Result<T, String> {
        let mut slot = self
            .browser
            .lock()
            .map_err(|_| "renderer browser lock is poisoned".to_string())?;
        if slot.is_none() {
            let config = self
                .config
                .lock()
                .map_err(|_| "renderer configuration lock is poisoned".to_string())?;
            *slot = Some(
                browsers_headless::HeadlessBrowser::new(browsers_headless::HeadlessConfig {
                    viewport_width: config.viewport_width,
                    viewport_height: config.viewport_height,
                    load_timeout_ms: config.load_timeout_ms,
                    wait_after_load_ms: config.wait_after_load_ms,
                })
                .map_err(|err| err.to_string())?,
            );
        }
        render(slot.as_ref().expect("browser was created just above"))
            .map_err(|err| err.to_string())
    }

    /// load_timeout resolves the deadline for one call, preferring the
    /// caller's override over the session default.
    fn load_timeout(&self, timeout_ms: Option<u64>) -> Result<u64, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "renderer configuration lock is poisoned".to_string())?;
        Ok(timeout_ms.unwrap_or(config.load_timeout_ms))
    }
}

impl Renderer for ServeRenderer {
    fn scrape(&self, url: &str, timeout_ms: Option<u64>) -> Result<ScrapeOutcome, String> {
        let deadline = self.load_timeout(timeout_ms)?;
        let result = self.with_browser(|browser| browser.scrape_with_timeout(url, deadline))?;
        Ok(ScrapeOutcome {
            url: result.url,
            title: result.title,
            html: result.html,
            timed_out: result.timed_out,
            duration_ms: result.duration_ms,
            memory_kb: result.memory_kb,
        })
    }

    fn screenshot(&self, url: &str, timeout_ms: Option<u64>) -> Result<ScreenshotOutcome, String> {
        let deadline = self.load_timeout(timeout_ms)?;
        let result =
            self.with_browser(|browser| browser.screenshot_bytes_with_timeout(url, deadline))?;
        Ok(ScreenshotOutcome {
            url: result.url,
            title: result.title,
            png: result.png,
            timed_out: result.timed_out,
            duration_ms: result.duration_ms,
            memory_kb: result.memory_kb,
        })
    }
}

/// RenderArgs are the arguments shared by the two render tools.
#[derive(Debug, Deserialize)]
struct RenderArgs {
    #[serde(default)]
    url: String,
    #[serde(default)]
    timeout_ms: Option<i64>,
}

/// parse_render_args decodes and validates tool arguments, returning the URL and
/// an optional per-call timeout. Failures name the field the model should fix.
fn parse_render_args(raw: &str) -> Result<(String, Option<u64>), String> {
    if raw.trim().is_empty() {
        return Err("missing required argument: url".to_string());
    }
    let value: serde_json::Value =
        serde_json::from_str(raw).map_err(|err| format!("invalid arguments: {err}"))?;
    if !value.is_object() {
        return Err("arguments must be an object".to_string());
    }
    let args: RenderArgs =
        serde_json::from_value(value).map_err(|err| format!("invalid arguments: {err}"))?;

    if args.url.trim().is_empty() {
        return Err("missing required argument: url".to_string());
    }
    let url = validate_url(&args.url)?;

    let timeout_ms = match args.timeout_ms {
        None => None,
        Some(value) if value < 0 => {
            return Err(format!("timeout_ms must not be negative, got {value}"))
        }
        Some(value) if value as u64 > MAX_TIMEOUT_MS => {
            return Err(format!(
                "timeout_ms must not exceed {MAX_TIMEOUT_MS}, got {value}"
            ))
        }
        Some(0) => None,
        Some(value) => Some(value as u64),
    };

    Ok((url, timeout_ms))
}

/// validate_url accepts only absolute http and https URLs. Anything else — a
/// relative path, a file:// URL, a javascript: URL — is refused.
fn validate_url(raw: &str) -> Result<String, String> {
    let parsed = url::Url::parse(raw).map_err(|err| format!("invalid url: {err}"))?;
    match parsed.scheme() {
        "http" | "https" => Ok(parsed.to_string()),
        other => Err(format!("unsupported scheme: {other}")),
    }
}

fn handle_scrape(renderer: std::sync::Arc<dyn Renderer>) -> ToolHandler {
    Box::new(move |raw: &str| match parse_render_args(raw) {
        Err(message) => ToolResult::failure(message),
        Ok((url, timeout_ms)) => match renderer.scrape(&url, timeout_ms) {
            Err(message) => ToolResult::failure(format!("scrape failed: {message}")),
            Ok(outcome) => ToolResult::success(payload(json!({
                "url": outcome.url,
                "title": outcome.title,
                "html": outcome.html,
                "timed_out": outcome.timed_out,
                "duration_ms": outcome.duration_ms,
                "memory_kb": outcome.memory_kb,
            }))),
        },
    })
}

fn handle_screenshot(renderer: std::sync::Arc<dyn Renderer>) -> ToolHandler {
    Box::new(move |raw: &str| match parse_render_args(raw) {
        Err(message) => ToolResult::failure(message),
        Ok((url, timeout_ms)) => match renderer.screenshot(&url, timeout_ms) {
            Err(message) => ToolResult::failure(format!("screenshot failed: {message}")),
            Ok(outcome) => {
                // The summary reports the PNG size, not its bytes: the image
                // itself travels as a separate base64 content block.
                let summary = json!({
                    "url": outcome.url,
                    "title": outcome.title,
                    "png_bytes": outcome.png.len(),
                    "timed_out": outcome.timed_out,
                    "duration_ms": outcome.duration_ms,
                    "memory_kb": outcome.memory_kb,
                });
                let encoded = base64::engine::general_purpose::STANDARD.encode(&outcome.png);
                ToolResult {
                    content: vec![
                        ContentItem::text(payload(summary)),
                        ContentItem::image(encoded, "image/png"),
                    ],
                    is_error: false,
                }
            }
        },
    })
}

fn handle_version() -> ToolHandler {
    Box::new(|_raw: &str| {
        ToolResult::success(payload(json!({
            "server": super::protocol::SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION"),
        })))
    })
}

/// payload renders a tool result as indented JSON. The result is a single string
/// inside a JSON frame, so its newlines are escaped on the wire and never split
/// the frame.
fn payload(value: Value) -> String {
    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|err| format!("could not encode result: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(raw: &str) -> Result<(String, Option<u64>), String> {
        parse_render_args(raw)
    }

    #[test]
    fn a_url_is_required() {
        assert_eq!(args("{}").unwrap_err(), "missing required argument: url");
        assert_eq!(args("").unwrap_err(), "missing required argument: url");
    }

    #[test]
    fn a_https_url_is_accepted() {
        let (url, timeout) = args(r#"{"url":"https://example.com/a?b=c"}"#).expect("valid");
        assert_eq!(url, "https://example.com/a?b=c");
        assert_eq!(timeout, None);
    }

    #[test]
    fn non_http_schemes_are_refused() {
        for raw in [
            r#"{"url":"file:///etc/passwd"}"#,
            r#"{"url":"javascript:alert(1)"}"#,
            r#"{"url":"ftp://example.com"}"#,
            r#"{"url":"/relative/path"}"#,
        ] {
            assert!(args(raw).is_err(), "{raw} should be refused");
        }
    }

    #[test]
    fn a_negative_timeout_is_refused() {
        let err = args(r#"{"url":"https://example.com","timeout_ms":-1}"#).unwrap_err();
        assert!(err.contains("must not be negative"), "{err}");
    }

    #[test]
    fn a_timeout_beyond_the_cap_is_refused_rather_than_wrapping() {
        // i64::MAX milliseconds would overflow the duration arithmetic and land
        // non-positive, silently disabling the deadline. It must be rejected.
        let err =
            args(r#"{"url":"https://example.com","timeout_ms":9223372036854775807}"#).unwrap_err();
        assert!(err.contains("must not exceed"), "{err}");
    }

    #[test]
    fn a_zero_timeout_means_use_the_server_default() {
        let (url, timeout) =
            args(r#"{"url":"https://example.com","timeout_ms":0}"#).expect("valid");
        // Url::parse normalises a bare host by adding the root path.
        assert_eq!(url, "https://example.com/");
        assert_eq!(timeout, None);
    }

    #[test]
    fn a_valid_timeout_is_carried_through() {
        let (_, timeout) =
            args(r#"{"url":"https://example.com","timeout_ms":5000}"#).expect("valid");
        assert_eq!(timeout, Some(5_000));
    }
}
