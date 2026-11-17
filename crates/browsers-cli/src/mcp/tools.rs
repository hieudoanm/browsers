//! Tool definitions and the tools/list result.
//!
//! Schemas are written out by hand because MCP clients read them to build the
//! tool descriptions a model sees.

use serde::Serialize;
use serde_json::{json, Value};

/// Tool names exposed over MCP. They carry a server prefix so a model can tell
/// which server produced a result when several servers share one session.
pub const TOOL_SCRAPE: &str = "browserverless_scrape";
pub const TOOL_SCREENSHOT: &str = "browserverless_screenshot";
pub const TOOL_VERSION: &str = "browserverless_version";

/// Annotations are the four MCP tool hints hosts use to decide whether a call
/// needs confirmation. Every tool declares all four explicitly so clients and
/// directories that require them never have to guess.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    pub read_only_hint: bool,
    pub destructive_hint: bool,
    pub idempotent_hint: bool,
    pub open_world_hint: bool,
}

impl Annotations {
    /// new builds the hint set in declaration order.
    pub const fn new(
        read_only_hint: bool,
        destructive_hint: bool,
        idempotent_hint: bool,
        open_world_hint: bool,
    ) -> Self {
        Annotations {
            read_only_hint,
            destructive_hint,
            idempotent_hint,
            open_world_hint,
        }
    }
}

/// RENDER_ANNOTATIONS is shared by scrape and screenshot: the call navigates
/// the browser to a remote URL and runs its scripts, so it is neither a pure
/// read nor guaranteed side-effect-free, but it never deletes data and it can
/// always reach the network.
const RENDER_ANNOTATIONS: Annotations = Annotations::new(false, false, false, true);

/// VERSION_ANNOTATIONS: reporting the binary version is a pure local read.
const VERSION_ANNOTATIONS: Annotations = Annotations::new(true, false, true, false);

/// Tool is one entry in the tools/list result.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    pub annotations: Annotations,
}

/// scrape_tool renders a URL and returns its HTML.
pub fn scrape_tool() -> Tool {
    Tool {
        name: TOOL_SCRAPE.to_string(),
        description: "Render a URL in the headless browser and return the full HTML document, its final URL, title, and render metrics.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": url_property(),
                "timeout_ms": timeout_property(),
            },
            "required": ["url"],
        }),
        annotations: RENDER_ANNOTATIONS,
    }
}

/// screenshot_tool renders a URL and returns a PNG screenshot.
pub fn screenshot_tool() -> Tool {
    Tool {
        name: TOOL_SCREENSHOT.to_string(),
        description: "Render a URL in the headless browser and return a PNG screenshot as an image, with the final URL, title, and render metrics.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "url": url_property(),
                "timeout_ms": timeout_property(),
            },
            "required": ["url"],
        }),
        annotations: RENDER_ANNOTATIONS,
    }
}

/// version_tool reports the binary version backing this server.
pub fn version_tool() -> Tool {
    Tool {
        name: TOOL_VERSION.to_string(),
        description: "Report the browserverless binary version backing this server.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
        }),
        annotations: VERSION_ANNOTATIONS,
    }
}

fn url_property() -> Value {
    json!({
        "type": "string",
        "description": "absolute http or https URL to render",
    })
}

fn timeout_property() -> Value {
    json!({
        "type": "integer",
        "description": "per-call load timeout in milliseconds; omit to use the server default",
    })
}

/// list_result builds the tools/list result, sorted by name so clients and tests
/// see a stable order.
pub fn list_result(tools: &[Tool]) -> Value {
    let mut sorted: Vec<&Tool> = tools.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    json!({ "tools": sorted })
}
