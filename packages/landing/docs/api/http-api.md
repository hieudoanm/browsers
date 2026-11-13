---
title: HTTP API
description: The browserverless serve endpoints, request/response shapes, and metadata headers.
order: 4
---

# HTTP API

`browserverless serve` exposes a small HTTP API backed by the same headless
Servo pipeline as the CLI. It is a thin layer — every request renders through
`HeadlessBrowser`; there is no custom HTML renderer.

```bash
browserverless serve --bind 127.0.0.1:8080
```

## Endpoints

| Method | Path                   | Body                     | Response                      |
| ------ | ---------------------- | ------------------------ | ----------------------------- |
| `GET`  | `/api/v1/health`       | —                        | `200 {"status":"ok"}`         |
| `GET`  | `/api/v1/version`      | —                        | `200` crate version string    |
| `GET`  | `/api/v1/openapi.json` | —                        | OpenAPI spec                  |
| `GET`  | `/docs`                | —                        | Interactive API docs page     |
| `POST` | `/api/v1/scrape`       | `{"url": "https://..."}` | `200` full HTML (`text/html`) |
| `POST` | `/api/v1/screenshot`   | `{"url": "https://..."}` | `200` PNG (`image/png`)       |

### Example

```bash
curl -X POST 'http://127.0.0.1:8080/api/v1/scrape' \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/"}'

curl -X POST 'http://127.0.0.1:8080/api/v1/screenshot' \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/"}' -o screenshot.png
```

The screenshot is rendered at the server viewport (`--width`/`--height`,
default 1280×720).

## Response headers

Both 200 scrape/screenshot responses carry request metadata:

```text
x-browserverless-url:          final URL after redirects
x-browserverless-title:        document.title (may be empty)
x-browserverless-load-status:  "ok" or "partial" (timed out mid-load)
x-browserverless-memory-kb:    peak RSS increase for the request
x-browserverless-duration-ms:  wall time for the request
```

Header values are ASCII-sanitized.

## Errors

Errors are JSON `{"error":"..."}`:

| Status | Meaning                                              |
| ------ | ---------------------------------------------------- |
| `400`  | Missing/invalid URL, bad JSON, or non-http(s) scheme |
| `404`  | Unknown path                                         |
| `405`  | Non-POST on scrape/screenshot                        |
| `500`  | Other render failure                                 |
| `504`  | Render timed out                                     |

Request bodies are capped at 64 KiB. Only `http` and `https` targets are
accepted — `file`, `data`, `javascript`, and custom schemes are rejected.

## Concurrency model

Servo initializes process-global state and `SoftwareRenderingContext` needs a
dedicated thread, so the server runs **one worker thread** that owns the single
`HeadlessBrowser` and processes requests serially. Throughput is one render at
a time; this is a deliberate MVP constraint, not a bug. Requests queue in the
accept loop.

> **Note:** Private-network/SSRF hardening (blocking localhost, RFC-1918,
> allowlists) is planned but not yet implemented — do not expose `serve` to
> untrusted networks.
