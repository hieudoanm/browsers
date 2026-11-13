# AGENTS.md — browsers-core

## Overview

Library crate that owns the Servo embedding boundary. Everything Servo-shaped
lives here; consumers (`browsers-headless`, `browsers-cli`) never
import `servo` types directly.

## Structure

- `src/lib.rs` — `BrowserError`, `BrowserConfig`, `BrowserContext`, `Page`,
  `PageDelegate`, `PageHandle` (single file by design; the Servo surface is
  small but subtle).

## Build

```bash
cargo build -p browsers-core
cargo test -p browsers-core
```

## Conventions

- Keep this crate free of CLI/HTTP concerns — no clap, no tiny_http, no
  process-level config. It is a library, not an application.
- Errors are explicit via `BrowserError` (`thiserror`); never `unwrap()` in
  library code paths.
- The initial navigation URL **must** be passed to `WebViewBuilder::url()` —
  do not "fix" this by calling `webview.load()` later; that race is documented
  in `https://hieudoanm.github.io/browsers/docs/project/architecture/`.
- Servo is process-global: assume exactly one `BrowserContext` exists per
  process. Tests that need a browser must share one test function.
