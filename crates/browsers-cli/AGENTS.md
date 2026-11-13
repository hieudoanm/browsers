# AGENTS.md — browsers-cli

## Overview

Binary crate producing the `browserverless` executable, plus the `serve` and
`mcp` library modules so the HTTP and MCP servers are testable without going
through `main`.

## Structure

- `src/main.rs` — clap `Cli`/`Commands`/`McpCommand` + dispatch. Keep the
  entrypoint thin: parse flags, build `HeadlessConfig`, call into
  `browsers-headless`/`serve`/`headed`.
- `src/headed.rs` — headed mode (winit + `WindowRenderingContext`).
- `src/serve.rs` — HTTP server on `tiny_http`: routing, JSON body parsing
  (64 KiB cap), `x-browserverless-*` metadata headers, request logging.
- `src/mcp/` — MCP server: `protocol.rs` (JSON-RPC types), `transport.rs`
  (`FrameReader`, 8 MiB cap, EOF-without-newline tolerance), `tools.rs`
  (`browserverless_scrape|screenshot|version`), `handlers.rs` (`Renderer`
  trait + `ServeRenderer` holding `Mutex<Option<HeadlessBrowser>>`).
- `tests/` — `serve_api.rs`, `mcp.rs`, `mcp_render.rs`; `tests/fixtures/` —
  HTML pages for manual runs (`make screenshot`, `make gui`).

## Build

```bash
cargo build -p browsers-cli   # binary: target/release/browserverless
cargo test -p browsers-cli
```

## Conventions

- `[[bin]] name = "browserverless"` — the binary name is a product contract;
  the package name is `browsers-cli`.
- MCP transport rules are contractual: stdout carries only protocol frames,
  diagnostics go to stderr; frames cap at 8 MiB; requests without `id` are
  notifications and must never be answered; tool failures return `isError`,
  protocol failures return JSON-RPC errors.
- One `HeadlessBrowser` per process — `ServeRenderer` builds it lazily behind
  a mutex; do not construct a second one per tool call (Servo panics on
  re-init).
- HTTP API: only `http`/`https` URLs, JSON error bodies, metadata headers on
  200s. New endpoints must preserve the `x-browserverless-*` header contract.
- Tests that need a real browser are consolidated into single integration
  tests per file (Servo is process-global).
