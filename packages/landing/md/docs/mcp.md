---
title: MCP server
description: Drive browserverless from AI agents over Model Context Protocol on stdio.
order: 5
---

# MCP server

`browserverless mcp serve` speaks the
[Model Context Protocol](https://modelcontextprotocol.io/) over **stdio**,
using newline-delimited JSON-RPC 2.0. It lets AI agents (Claude Desktop,
Cursor, custom clients) screenshot and scrape pages through the real Servo
engine.

```bash
browserverless mcp serve [--width 1280] [--height 720] [--timeout 30000]
```

Requests are read from stdin; every reply is a single line on stdout;
diagnostics go to stderr so they can never corrupt the protocol stream.

## Tools

| Tool                        | Arguments                    | Result                                                              |
| --------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| `browserverless_scrape`     | `url`, optional `timeout_ms` | Text block: final URL, title, full HTML, request cost               |
| `browserverless_screenshot` | `url`, optional `timeout_ms` | Text summary + base64 `image/png` content block                     |
| `browserverless_version`    | none                         | Server name and crate version                                       |

`timeout_ms` overrides `--timeout` for that call; `0` means "use the server
default". Missing `url`, negative/oversized `timeout_ms`, and non-`http(s)`
schemes come back as tool errors (`isError: true`) so the model can correct
the call.

## Protocol rules

- A request **without** an `id` is a notification — never answered.
- Frames are capped at **8 MiB**; an oversized frame is discarded up to the
  next newline rather than executed.
- `tools/list` is sorted by name; `tools/call` failures are tool-level errors,
  while unknown methods and malformed frames are JSON-RPC errors.

## One browser per process

Servo's options are process-global and panic on second initialization, so the
server builds **one** `HeadlessBrowser` lazily on first tool call and reuses
it. Each call gets a fresh page inside it, serialized by a mutex. This keeps
MCP sessions cheap after the first render.

## Client configuration

For stdio-based MCP clients, register the server command as:

```json
{
  "mcpServers": {
    "browserverless": {
      "command": "browserverless",
      "args": ["mcp", "serve"]
    }
  }
}
```

> **Warning:** Unlike `serve`, MCP mode does not restrict targets to public
> addresses — a client-supplied URL is rendered as asked, including private
> IPs. Only give MCP access to trusted agents.
