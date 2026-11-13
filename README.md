# Browserverless

A lightweight browser runtime built on the **Servo** engine, with one Rust codebase and
two execution modes:

- **Headed** — interactive desktop window (winit + Servo `WindowRenderingContext`).
- **Headless** — offscreen rendering and PNG screenshots (Servo `SoftwareRenderingContext`),
  no window, desktop session, or monitor required.

The project intentionally does **not** roll its own HTML/CSS/JS engine and never embedded
Chromium/WebKit/Gecko. Servo owns the web platform; this repo is a thin embedder shell.

## Workspace

| Crate                      | Responsibility                                                        |
| -------------------------- | --------------------------------------------------------------------- |
| `crates/browsers-core`     | Servo integration boundary: `BrowserContext`, `Page`, rendering setup |
| `crates/browsers-headless` | Offscreen execution: load → wait → screenshot / HTML scrape           |
| `crates/browsers-cli`      | clap CLI + `serve` HTTP API + `mcp serve` MCP server                  |

The bin is `browserverless`; the compile package is `browsers-cli`.

Nine further `browsers-*` crates (`-css`, `-dom`, `-gui`, `-html`,
`-layout`, `-network`, `-paint`, `-renderer`, `-style`) are an experimental
pure-Rust engine prototype — intentionally **not** workspace members; see
the [roadmap docs](https://hieudoanm.github.io/browsers/docs/project/roadmap/).

`packages/landing` is the Next.js static site (marketing page + docs) deployed
to GitHub Pages; `assets/` holds the canonical branding SVGs.

## Build

```bash
cargo build                    # workspace debug build
cargo build --release

# or via pnpm/turbo (also builds the landing page)
pnpm install
pnpm build
```

## Run

```bash
# Screenshot a URL (headless)
cargo run -p browsers-cli -- screenshot \
  https://example.com --output /tmp/example.png \
  --width 1280 --height 720 --timeout 60000

# Scrape the full HTML of a URL to a file (headless)
cargo run -p browsers-cli -- scrape \
  https://example.com --output /tmp/example.html --timeout 60000

# Headless load-and-exit
cargo run -p browsers-cli -- headless https://example.com --timeout 30000

# Measure peak RSS + duration for a URL (headless)
cargo run -p browsers-cli -- memory \
  https://example.com --timeout 60000 --observe 8000

# Serve the headless rendering web API on http://127.0.0.1:8080
cargo run -p browsers-cli -- serve --bind 127.0.0.1:8080

# Serve the Model Context Protocol on stdio
cargo run -p browsers-cli -- mcp serve

# Headed window (currently loads a hardcoded https://example.com)
cargo run -p browsers-cli -- open
```

CLI flags: `screenshot <url> [--output screenshot.png] [--width 1280]
[--height 720] [--timeout 30000]`; `scrape <url> [-o page.html] [--timeout 30000]`
(omit `-o` to print to stdout); `memory <url> [--timeout 30000] [--observe 5000]`;
`serve [--bind 127.0.0.1:8080] [--port 8080] [--width 1280] [--height 720]
[--timeout 30000]` (`--port` overrides the port in `--bind`).
`mcp serve [--width 1280] [--height 720] [--timeout 30000]`.
The URL is positional and must be absolute (`file://`, `http://`, `https://`).
Use a release build for meaningful timings (`cargo build --release`).

On startup the server prints a boxed banner with the `http://ip:port` URL and the
raw `ip:port`, using the machine's external (LAN) IP when discoverable. Each
request is logged to stdout as `[timestamp] req=N <METHOD> <path> -> <status>
(<duration>ms)`, e.g.

```text
[2026-08-28 09:10:42] req=1 POST /api/v1/scrape -> 200 (459ms)
```

### Server API

```text
GET  /api/v1/health            -> 200 {"status":"ok"}
GET  /api/v1/version           -> 200 <crate version>
POST /api/v1/scrape            -> 200 <full HTML of url> (text/html)
POST /api/v1/screenshot        -> 200 <PNG of url> (image/png)
                                  body: {"url": "https://..."}
```

The scrape body is the page's full HTML; the screenshot body is the page rendered
at the serve viewport (`--width`/`--height`, default 1280x720). Both carry
metadata about the URL and the request cost as response headers:

```
x-browserverless-url:          final URL after redirects (location.href)
x-browserverless-title:        page title (document.title)
x-browserverless-load-status:  "ok" or "partial" (timed out before full load)
x-browserverless-memory-kb:    peak RSS increase attributed to the request
x-browserverless-duration-ms:  wall time for the request
```

Only `http`/`https` targets are allowed (400 otherwise); missing `url` or invalid
JSON body is 400; a non-POST on `/api/v1/scrape` or `/api/v1/screenshot` is 405;
unknown paths are 404; render timeouts are 504. Details in the [architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/)
§Server Mode.

### MCP

`mcp serve` speaks MCP over stdio using newline-delimited JSON-RPC 2.0. Requests
are read from stdin; every reply is written as a single line to stdout, and
diagnostics go to stderr so they can never corrupt the protocol stream.

| Tool                        | Arguments                    | Result                                                              |
| --------------------------- | ---------------------------- | ------------------------------------------------------------------- |
| `browserverless_scrape`     | `url`, optional `timeout_ms` | text block with the page's final URL, title, HTML, and request cost |
| `browserverless_screenshot` | `url`, optional `timeout_ms` | a text summary plus a base64 `image/png` block                      |
| `browserverless_version`    | none                         | the server name and crate version                                   |

`timeout_ms` overrides `--timeout` for that call. Zero means "use the server
default". A missing `url`, a negative or oversized `timeout_ms`, and any
non-`http`/`https` scheme are returned to the model as tool errors
(`isError: true`) so it can correct the call, rather than as JSON-RPC errors.
Only `http` and `https` are accepted, matching the server API; a client-supplied
URL is not otherwise restricted, so this mode renders private addresses if asked
to. Frames are capped at 8 MiB and the reader resynchronises on the next
newline, so an oversized frame cannot exhaust memory or be executed as a second
command. A JSON-RPC request without an `id` is a notification and is never
answered.

Servo initialises process-wide state that panics if a second browser is
constructed, so the server builds one browser on first use and gives each tool
call a fresh page within it. Details in the [architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/) §MCP.

## Test / check

```bash
cargo test --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

CI runs these gates in `.github/workflows/ci.yaml` (Rust check + release
binaries, plus the landing build/deploy to GitHub Pages).

## Docs

- `CONTRIBUTING.md` — build/run/test, conventions, troubleshooting, Servo upgrade steps.
- `AGENTS.md` — the project spec (mission, phase plan, conventions).
- `packages/landing/md/docs/` — published docs site (deployed to GitHub Pages at
  `https://hieudoanm.github.io/browsers/docs/`): overview, getting started, CLI
  reference, HTTP API, MCP, architecture (incl. Servo integration notes),
  packaging/releases, and roadmap.

## Known constraints (summary)

- Servo initializes **once per process** (`opts`/`prefs` are process-global).
- Use one page per `BrowserContext`; multi-page in one context is unreliable today.
- Initial navigation must be passed via `WebViewBuilder::url()`, not `webview.load()`
  afterwards (a load race produced the all-white-screenshot bug; now fixed with a
  regression test). Details in the [architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/) §Servo Integration Notes.

## License

GPL-3.0 (see `LICENSE`).