//! JSON-RPC 2.0 and MCP wire types.
//!
//! Field names follow the MCP specification rather than Rust's defaults, so
//! `isError` and `inputSchema` reach a client unchanged.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// JSONRPC_VERSION is the only protocol version this server accepts.
pub const JSONRPC_VERSION: &str = "2.0";

/// PROTOCOL_VERSION is the MCP revision this server implements.
pub const PROTOCOL_VERSION: &str = "2025-11-25";

/// SERVER_NAME identifies this server during initialize.
pub const SERVER_NAME: &str = "browserverless-mcp";

/// JSON-RPC 2.0 error codes used by the server.
pub const ERR_CODE_PARSE: i32 = -32700;
pub const ERR_CODE_INVALID_REQUEST: i32 = -32600;
pub const ERR_CODE_METHOD_NOT_FOUND: i32 = -32601;
pub const ERR_CODE_INVALID_PARAMS: i32 = -32602;
pub const ERR_CODE_INTERNAL_ERROR: i32 = -32603;

/// Request is an incoming JSON-RPC message. A missing or null id makes it a
/// notification, which is never answered.
#[derive(Debug, Deserialize)]
pub struct Request {
    #[serde(default)]
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub params: Option<Value>,
}

impl Request {
    /// is_notification reports whether this frame expects no reply.
    pub fn is_notification(&self) -> bool {
        matches!(self.id, None | Some(Value::Null))
    }
}

/// Response is an outgoing JSON-RPC reply. Exactly one of result and error is
/// populated; `isError` marks a tool that ran but failed.
#[derive(Debug, Serialize)]
pub struct Response {
    pub jsonrpc: &'static str,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorObject>,
}

/// ErrorObject is the JSON-RPC error member.
#[derive(Debug, Serialize)]
pub struct ErrorObject {
    pub code: i32,
    pub message: String,
}

/// ToolResult is the payload of a tools/call result. `isError` is omitted when
/// false so a successful result stays compact.
#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<ContentItem>,
    #[serde(rename = "isError", skip_serializing_if = "is_false")]
    pub is_error: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

/// ContentItem is one piece of tool output: a text block, or an image whose
/// data holds standard base64 bytes.
#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

impl ContentItem {
    /// text builds a text content block.
    pub fn text(body: impl Into<String>) -> Self {
        ContentItem {
            kind: "text".to_string(),
            text: Some(body.into()),
            data: None,
            mime_type: None,
        }
    }

    /// image builds an image content block from base64 bytes. The PNG travels
    /// here rather than inlined as text so a model is not made to read megabytes
    /// of base64.
    pub fn image(base64: impl Into<String>, mime_type: impl Into<String>) -> Self {
        ContentItem {
            kind: "image".to_string(),
            text: None,
            data: Some(base64.into()),
            mime_type: Some(mime_type.into()),
        }
    }
}

impl ToolResult {
    /// success builds a successful result carrying text.
    pub fn success(text: impl Into<String>) -> Self {
        ToolResult {
            content: vec![ContentItem::text(text)],
            is_error: false,
        }
    }

    /// failure builds a failed result carrying text. Tool failures are reported
    /// this way rather than as JSON-RPC errors so the model can see and react.
    pub fn failure(text: impl Into<String>) -> Self {
        ToolResult {
            content: vec![ContentItem::text(text)],
            is_error: true,
        }
    }
}

/// ToolCallParams are the params of a tools/call request. `arguments` is the
/// tool's own argument object, kept as a Value so a handler receives exactly
/// what the client sent.
#[derive(Debug, Deserialize)]
pub struct ToolCallParams {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub arguments: Option<Value>,
}

impl ToolCallParams {
    /// arguments_json returns the arguments as text for a handler. Absent
    /// arguments become an empty object so a handler sees a well-formed value.
    pub fn arguments_json(&self) -> String {
        match &self.arguments {
            None | Some(Value::Null) => "{}".to_string(),
            Some(value) => value.to_string(),
        }
    }
}

/// object_or_empty decodes call params, treating an absent or JSON-null value as
/// an empty object. A non-object value is rejected, because decoding it into
/// ToolCallParams would silently drop the fields a caller meant to send.
pub fn object_or_empty(params: &Option<Value>) -> Result<ToolCallParams, String> {
    let value = match params {
        None | Some(Value::Null) => json!({}),
        Some(value) => value.clone(),
    };
    if !value.is_object() {
        return Err("params must be an object".to_string());
    }
    serde_json::from_value(value).map_err(|err| err.to_string())
}

/// initialize_result builds the initialize reply.
///
/// This server speaks exactly one revision, so it always answers with
/// PROTOCOL_VERSION whether or not the client asked for it. A client that
/// requested something else is expected to disconnect after seeing the
/// mismatch, which is what the specification prescribes.
pub fn initialize_result(_params: &Option<Value>) -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") },
    })
}

/// error_response builds a JSON-RPC error reply for id.
pub fn error_response(id: Option<Value>, code: i32, message: &str) -> Response {
    Response {
        jsonrpc: JSONRPC_VERSION,
        id,
        result: None,
        error: Some(ErrorObject {
            code,
            message: message.to_string(),
        }),
    }
}

/// success_response builds a JSON-RPC success reply for id.
pub fn success_response(id: Option<Value>, result: Value) -> Response {
    Response {
        jsonrpc: JSONRPC_VERSION,
        id,
        result: Some(result),
        error: None,
    }
}
