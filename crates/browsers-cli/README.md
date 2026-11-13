# browsers-cli

The `browserverless` binary — clap CLI plus the HTTP server (`serve`) and MCP
server (`mcp serve`) library modules. Compile package `browsers-cli`;
binary name `browserverless`.

## Layout

- `src/main.rs` — clap definition + subcommand dispatch (`open`, `headless`,
  `screenshot`, `scrape`, `memory`, `serve`, `mcp serve`).
- `src/headed.rs` — winit window + `WindowRenderingContext` headed mode.
- `src/serve.rs` — `tiny_http` server: `/api/v1/health`, `/version`,
  `/openapi.json`, `/docs`, `POST /api/v1/scrape`, `POST /api/v1/screenshot`.
- `src/mcp/` — newline-delimited JSON-RPC 2.0 over stdio:
  `protocol.rs` (framing), `transport.rs` (8 MiB frame cap), `tools.rs`
  (tool defs), `handlers.rs` (`Renderer` trait + `ServeRenderer`).
- `tests/` — integration tests (`serve_api.rs`, `mcp.rs`, `mcp_render.rs`)
  plus `fixtures/` HTML pages used by `make screenshot`/`make gui`.

## Commands

```bash
browserverless screenshot <url> [--output f.png] [--width 1280] [--height 720] [--timeout 30000]
browserverless scrape <url> [-o page.html] [--timeout 30000]
browserverless headless <url> [--timeout 30000] [--wait 3000]
browserverless memory <url> [--timeout 30000] [--observe 5000]
browserverless serve [--bind 127.0.0.1:8080] [--port 8080] [--timeout 30000]
browserverless mcp serve [--timeout 30000]
browserverless open    # headed window (currently example.com)
```

## Notes

- Servo is process-global → the server and MCP server each own exactly one
  `HeadlessBrowser`, built lazily and serialized.
- URL policy: only absolute `http`/`https`/`file` targets; the HTTP API
  additionally restricts to `http`/`https`.
