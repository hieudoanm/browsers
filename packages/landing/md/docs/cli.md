---
title: CLI reference
description: Every browserverless subcommand, its flags, and what it prints.
order: 3
---

# CLI reference

The binary is `browserverless`. Flags go before the positional URL:

```bash
browserverless screenshot --timeout 5000 https://example.com
```

## `screenshot`

Render a URL to a PNG.

```bash
browserverless screenshot <url> \
  [--output screenshot.png] [--width 1280] [--height 720] [--timeout 30000]
```

| Flag        | Default          | Meaning                                  |
| ----------- | ---------------- | ---------------------------------------- |
| `--output`  | `screenshot.png` | PNG output path                          |
| `--width`   | `1280`           | Viewport width in CSS pixels             |
| `--height`  | `720`            | Viewport height in CSS pixels            |
| `--timeout` | `30000`          | Load timeout in milliseconds             |

## `scrape`

Dump the fully rendered HTML of a page.

```bash
browserverless scrape <url> [-o page.html] [--timeout 30000]
```

Omit `-o` to print the HTML to stdout.

## `headless`

Load a URL, wait for render, and exit — the minimal smoke test.

```bash
browserverless headless <url> [--width 1280] [--height 720]
                    [--timeout 30000] [--wait 3000]
```

## `memory`

Measure peak RSS growth and wall time for a render — useful for capacity
planning.

```bash
browserverless memory <url> [--timeout 30000] [--observe 5000]
```

## `serve`

Start the HTTP rendering API. See [HTTP API](http-api).

```bash
browserverless serve [--bind 127.0.0.1:8080] [--port 8080]
                     [--width 1280] [--height 720] [--timeout 30000]
```

`--port` overrides the port inside `--bind`. On startup the server prints a
banner with the LAN URL, then logs one line per request:

```text
[2026-11-13 09:10:42] req=1 POST /api/v1/scrape -> 200 (459ms)
```

## `mcp serve`

Start the MCP server over stdio. See [MCP server](mcp).

```bash
browserverless mcp serve [--width 1280] [--height 720] [--timeout 30000]
```

## `open`

Open an interactive headed window (winit + Servo
`WindowRenderingContext`). Currently loads `https://example.com`.

```bash
browserverless open
```

## Exit behavior

All rendering commands share the same pipeline: parse the URL, spin up one
`HeadlessBrowser`, navigate, wait for `LoadStatus::Complete` (up to
`--timeout`), let layout settle for a short window, then produce output. A
non-absolute URL is rejected before Servo is touched.
