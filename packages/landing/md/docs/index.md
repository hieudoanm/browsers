---
title: Overview
description: What browserverless is and how the pieces fit together.
order: 1
---

# Browserverless documentation

**browserverless** is a lightweight browser runtime built on the
[Servo](https://servo.org/) engine, shipped as a single Rust binary. It does
not wrap Chromium, WebKit, or Gecko — Servo owns the web platform (HTML, CSS,
JavaScript) and this project is a thin embedder shell around it.

## What it does

- **Headless screenshots** — render any `http(s)` URL to a PNG with no window,
  desktop session, or monitor.
- **Scraping** — extract the fully rendered DOM of a page after JavaScript has
  run.
- **HTTP API** — `POST /api/v1/screenshot` and `POST /api/v1/scrape` behind a
  small `tiny_http` server.
- **MCP server** — the same engine exposed to AI agents as
  `browserverless_scrape`, `browserverless_screenshot`, and
  `browserverless_version` tools over stdio JSON-RPC.
- **Headed mode** — an interactive winit window when you need to watch the
  page render.

## Modes

| Mode      | Entry point                  | Use it for                                  |
| --------- | ---------------------------- | ------------------------------------------- |
| CLI       | `browserverless <command>`   | One-off screenshots, scrapes, memory probes |
| Server    | `browserverless serve`       | An HTTP rendering API                       |
| MCP       | `browserverless mcp serve`   | AI agents (Claude, Cursor, custom clients)  |
| Headed    | `browserverless open`        | Interactive browsing                         |

## Workspace

| Crate                     | Responsibility                                             |
| ------------------------- | ---------------------------------------------------------- |
| `browsers-core`     | Servo integration boundary: `BrowserContext`, `Page`       |
| `browsers-headless` | Offscreen rendering: load → wait → screenshot / HTML dump  |
| `browsers-cli`      | clap CLI, HTTP server, and MCP server; bin `browserverless` |

The remaining `browsers-*` crates under `crates/` (`-css`, `-dom`,
`-gui`, `-html`, `-layout`, `-network`, `-paint`, `-renderer`, `-style`) are an
experimental pure-Rust engine kept for research. They are **not** workspace
members and are not compiled — see the [roadmap](roadmap).

## Documentation map

- [Getting started](getting-started) — install, build, first screenshot
- [CLI reference](cli) — every subcommand and flag
- [HTTP API](http-api) — endpoints, headers, error codes
- [MCP server](mcp) — tools, transport, protocol rules
- [Architecture](architecture) — how Servo is embedded and the hard constraints
- [Roadmap](roadmap) — phase status and known limitations
