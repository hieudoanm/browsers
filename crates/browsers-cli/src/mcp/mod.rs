//! Model Context Protocol server exposing browserverless rendering as tools.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio: one frame per
//! line, diagnostics to stderr, and nothing but frames on stdout. Rendering is
//! delegated to the `headless` crate so this module holds protocol concerns
//! only and never talks to Servo directly.

pub mod handlers;
mod http;
pub mod protocol;
pub mod tools;
pub mod transport;

use std::io::Write;

use protocol::{Request, ToolResult};
use tools::Tool;

/// Runs the MCP server until stdin reaches EOF, which is how a client signals
/// shutdown. `diagnostics` receives notification notices and write failures;
/// stdout stays reserved for JSON-RPC frames.
pub fn run(
    width: u32,
    height: u32,
    timeout_ms: u64,
    diagnostics: &mut dyn Write,
) -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    let mut server = Server::with_diagnostics(&mut stdout, Diagnostics::Stream(diagnostics));
    server.register_tools(width, height, timeout_ms);
    server.serve(std::io::stdin())
}

/// Runs the MCP server over streamable HTTP on `addr` until the process
/// exits. Requests are handled sequentially on the accept thread because the
/// Servo renderer is thread-affine; one `HeadlessBrowser` is shared across
/// requests. When `token` is set, requests must carry
/// `Authorization: Bearer <token>`.
pub fn run_http(
    addr: &str,
    token: Option<String>,
    width: u32,
    height: u32,
    timeout_ms: u64,
) -> std::io::Result<()> {
    let mut out: Vec<u8> = Vec::new();
    let mut server = Server::with_diagnostics(&mut out, Diagnostics::Sink);
    let renderer = handlers::ServeRenderer::new(width, height, timeout_ms);
    for (tool, handler) in handlers::register(renderer) {
        server.add_tool(tool, handler);
    }
    http::serve_seq(addr, token, move |body: &str| {
        server
            .response_for(body.as_bytes())
            .and_then(|r| serde_json::to_string(&r).ok())
    })
}

/// ToolHandler executes one tool call. It receives the raw arguments and returns
/// a tool result rather than an error, so a model can see and react to a failure.
pub type ToolHandler = Box<dyn Fn(&str) -> ToolResult>;

/// Diagnostics receives notification notices and write failures. Owning the
/// sink rather than borrowing it keeps Server free of a lifetime parameter, so
/// tests can construct one inline.
pub enum Diagnostics<'a> {
    /// Discarded, which is what tests that do not assert on diagnostics want.
    Sink,
    /// Written to the caller's stream; stdout is never valid here.
    Stream(&'a mut dyn Write),
}

impl Write for Diagnostics<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Diagnostics::Sink => Ok(buf.len()),
            Diagnostics::Stream(stream) => stream.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Diagnostics::Sink => Ok(()),
            Diagnostics::Stream(stream) => stream.flush(),
        }
    }
}

/// Server dispatches MCP requests to registered tools and writes frames to
/// output. The input stream is supplied per call to `serve`, so one server can
/// be driven repeatedly in tests.
pub struct Server<'a> {
    output: &'a mut dyn Write,
    diagnostics: Diagnostics<'a>,
    tools: Vec<Tool>,
    handlers: Vec<(String, ToolHandler)>,
}

impl<'a> Server<'a> {
    /// new returns a Server writing frames to output, discarding diagnostics.
    /// Tests supply a buffer so the full protocol runs without touching the
    /// process stdio. The writer is borrowed, so it must outlive the server.
    pub fn new(output: &'a mut dyn Write) -> Server<'a> {
        Server {
            output,
            diagnostics: Diagnostics::Sink,
            tools: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// with_diagnostics is new with an explicit diagnostics sink.
    pub fn with_diagnostics(output: &'a mut dyn Write, diagnostics: Diagnostics<'a>) -> Server<'a> {
        Server {
            output,
            diagnostics,
            tools: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// add_tool registers a tool and its handler, replacing an entry with the
    /// same name.
    pub fn add_tool(&mut self, tool: Tool, handler: ToolHandler) {
        let name = tool.name.clone();
        match self.tools.iter_mut().find(|existing| existing.name == name) {
            Some(existing) => *existing = tool,
            None => self.tools.push(tool),
        }
        match self
            .handlers
            .iter_mut()
            .find(|(registered, _)| *registered == name)
        {
            Some((_, existing)) => *existing = handler,
            None => self.handlers.push((name, handler)),
        }
    }

    /// register_tools adds the browserverless tool surface, rendered in-process
    /// through the Servo-backed headless crate.
    pub fn register_tools(&mut self, width: u32, height: u32, timeout_ms: u64) {
        let renderer = handlers::ServeRenderer::new(width, height, timeout_ms);
        for (tool, handler) in handlers::register(renderer) {
            self.add_tool(tool, handler);
        }
    }

    /// serve reads and dispatches frames until EOF. A clean EOF is success.
    pub fn serve(&mut self, input: impl std::io::Read) -> std::io::Result<()> {
        let mut buffered = std::io::BufReader::new(input);
        let mut reader = transport::FrameReader::new(&mut buffered);
        loop {
            match reader.next_frame() {
                transport::Frame::Message(line) => self.handle_message(line.as_bytes()),
                transport::Frame::TooLong => {
                    self.write(protocol::error_response(
                        None,
                        protocol::ERR_CODE_PARSE,
                        "parse error: frame too large",
                    ));
                }
                transport::Frame::Eof => return Ok(()),
                transport::Frame::Error(err) => return Err(err),
            }
        }
    }

    /// handle_message decodes one frame and dispatches it, writing the reply.
    /// An undecodable frame produces a parse error. A notification — a frame
    /// with no id — is never answered whatever the method, because answering
    /// one desynchronises the client. That check precedes the version check so
    /// a notification which is also malformed stays silent.
    fn handle_message(&mut self, raw: &[u8]) {
        if let Some(response) = self.response_for(raw) {
            self.write(response);
        }
    }

    /// response_for returns the reply frame for one request, or `None` for
    /// notifications. Shared by the stdio loop (write) and the HTTP transport
    /// (serialize).
    pub fn response_for(&mut self, raw: &[u8]) -> Option<protocol::Response> {
        let request: Request = match serde_json::from_slice(raw) {
            Ok(request) => request,
            Err(err) => {
                return Some(protocol::error_response(
                    None,
                    protocol::ERR_CODE_PARSE,
                    &format!("parse error: {err}"),
                ));
            }
        };

        if request.is_notification() {
            let _ = writeln!(
                self.diagnostics,
                "ignoring notification for method {}",
                request.method
            );
            return None;
        }
        if request.jsonrpc != protocol::JSONRPC_VERSION {
            return Some(protocol::error_response(
                request.id.clone(),
                protocol::ERR_CODE_INVALID_REQUEST,
                "invalid jsonrpc version",
            ));
        }

        let id = request.id.clone();
        match request.method.as_str() {
            "initialize" => Some(protocol::success_response(
                id,
                protocol::initialize_result(&request.params),
            )),
            "ping" => Some(protocol::success_response(id, serde_json::json!({}))),
            "tools/list" => {
                let mut value = tools::list_result(&self.tools);
                if let Some(list) = value.get_mut("tools").and_then(|t| t.as_array_mut()) {
                    for tool in list {
                        if tool.get("title").is_none() {
                            if let Some(name) = tool.get("name").and_then(|n| n.as_str()) {
                                tool["title"] = serde_json::json!(tools::titleize(name));
                            }
                        }
                    }
                }
                Some(protocol::success_response(id, value))
            }
            "tools/call" => self.call_response(id, &request.params),
            other => Some(protocol::error_response(
                id,
                protocol::ERR_CODE_METHOD_NOT_FOUND,
                &format!("method not found: {other}"),
            )),
        }
    }

    /// call_response decodes the call params and runs the named handler,
    /// returning the reply frame. Absent or null params are treated as an
    /// empty object so a call with no params names no tool rather than
    /// failing to decode.
    fn call_response(
        &mut self,
        id: Option<serde_json::Value>,
        params: &Option<serde_json::Value>,
    ) -> Option<protocol::Response> {
        let call: protocol::ToolCallParams = match protocol::object_or_empty(params) {
            Ok(call) => call,
            Err(message) => {
                return Some(protocol::error_response(
                    id,
                    protocol::ERR_CODE_INVALID_PARAMS,
                    &format!("invalid params: {message}"),
                ));
            }
        };

        let handler = self
            .handlers
            .iter()
            .find(|(name, _)| *name == call.name)
            .map(|(_, handler)| handler);

        match handler {
            None => Some(protocol::error_response(
                id,
                protocol::ERR_CODE_METHOD_NOT_FOUND,
                &format!("tool not found: {}", call.name),
            )),
            Some(handler) => {
                let result = handler(&call.arguments_json());
                match serde_json::to_value(result) {
                    Ok(value) => Some(protocol::success_response(id, value)),
                    Err(err) => Some(protocol::error_response(
                        id,
                        protocol::ERR_CODE_INTERNAL_ERROR,
                        &format!("could not encode tool result: {err}"),
                    )),
                }
            }
        }
    }

    /// write emits one compact frame. Responses are marshalled compactly so a
    /// reply never spans lines, which would desynchronise a line-oriented client.
    fn write(&mut self, response: protocol::Response) {
        let encoded = match serde_json::to_string(&response) {
            Ok(encoded) => encoded,
            Err(err) => {
                let _ = writeln!(self.diagnostics, "could not encode response: {err}");
                return;
            }
        };
        if let Err(err) = writeln!(self.output, "{encoded}") {
            let _ = writeln!(self.diagnostics, "could not write response: {err}");
        }
    }
}
