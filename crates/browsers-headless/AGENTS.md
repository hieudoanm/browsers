# AGENTS.md — browsers-headless

## Overview

Library crate for headless (offscreen) rendering. Wraps
`browsers-core`'s `BrowserContext`/`Page` with wait-for-load, settle
delays, and the result types consumed by the CLI's HTTP server and MCP server.

## Structure

- `src/lib.rs` — `HeadlessError`, `HeadlessConfig`, `HeadlessBrowser`,
  `ScrapeResult`, `ScreenshotResult`.
- `tests/headless_rendering.rs` — end-to-end pipeline test (single test:
  Servo is process-global).

## Build

```bash
cargo build -p browsers-headless
cargo test -p browsers-headless
```

## Conventions

- Library only — no clap, no sockets. HTTP/MCP concerns live in
  `browsers-cli`.
- Errors via `HeadlessError` (`thiserror`); `BrowserError` converts via `#[from]`.
- `render_url_best_effort` is the shared internal path for scrape/screenshot —
  route new render consumers through it rather than duplicating the
  navigate→wait→settle sequence.
- Do not add a second `BrowserContext` — one per process (see
  `browsers-core` AGENTS.md).
