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

/// Tool is one entry in the tools/list result.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
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
