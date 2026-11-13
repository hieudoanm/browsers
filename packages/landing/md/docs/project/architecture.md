---
title: Architecture
description: How browserverless embeds Servo — the workspace, the render pipeline, and the constraints that shape it.
order: 6
---

# Architecture

This is a **Servo-embedding** project. The custom HTML/CSS/layout/renderer
crates from an earlier prototype (`browsers-css`, `browsers-dom`, `browsers-gui`,
`browsers-html`, `browsers-layout`, `browsers-network`, `browsers-paint`,
`browsers-renderer`, `browsers-style`) are **not compiled** — they are not
members of the Cargo workspace.

## Mission

One Rust codebase, one browser engine (Servo), two execution modes:

- **Headed** — interactive desktop window via winit + Servo
  `WindowRenderingContext`.
- **Headless** — offscreen rendering via Servo `SoftwareRenderingContext`,
  producing PNG screenshots. Also serves as the engine for server mode (HTTP
  rendering).

The project intentionally does **not** roll its own HTML/CSS/JS engine and
never embeds Chromium/WebKit/Gecko. Servo owns the web platform; this repo is a
thin embedder shell.

## Workspace Layout

```text
.
├── Cargo.toml                     # workspace: browsers-core, browsers-headless, browsers-cli
├── crates/
│   ├── browsers-core/             # Servo integration boundary (library)
│   ├── browsers-headless/         # offscreen rendering / screenshots (library)
│   ├── browsers-cli/              # clap CLI + serve + mcp; bin "browserverless"
│   └── browsers-{css,dom,gui,html,layout,network,paint,renderer,style}/
│                                 # experimental engine prototype (not workspace members)
├── crates/browsers-cli/tests/fixtures/rendering/mvp.html
│                                 # golden-geometry acceptance fixture
├── packages/landing/              # this documentation site (Next.js static export)
├── assets/branding/               # canonical logo.svg / og.svg + generated PNGs
└── docker/                        # alternate Dockerfile (port 10000)
```

## Crate Responsibilities

| Crate               | Responsibility                                                                                                                     |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| `browsers-core`     | Owns the Servo boundary: `BrowserContext`, `Page`, `PageDelegate`, rendering setup. Knows nothing about terminals or HTTP.         |
| `browsers-headless` | Owns offscreen execution: `HeadlessBrowser`, `HeadlessConfig`, wait-for-load/render, `screenshot_url`. Calls `browsers-core` only. |
| `browsers-cli`      | Owns command parsing (clap), the HTTP server, and the MCP server. Bin name is `browserverless`; compile package is `browsers-cli`. |

Dependency direction is strictly downwards:
`browsers-cli → browsers-headless → browsers-core → servo`.

## Core Domain Model

```text
HeadlessBrowser (browsers-headless)
  └── BrowserContext (browsers-core)      # owns Servo instance + RenderingContext
        └── Page (browsers-core)
              ├── WebView + WebViewDelegate (servo)
              └── PageHandle               # loaded / frames state shared with delegate
```

- `BrowserContext::new` builds a `SoftwareRenderingContext` sized to the viewport,
  then a `Servo` instance via `ServoBuilder::default().preferences(prefs).build()`.
- `BrowserContext::new_page(url)` builds a `WebView` with the URL supplied through
  `WebViewBuilder::url(...)`. **The initial URL must be set in the builder — see
  [Servo Integration Notes §1](#1-initial-navigation-must-be-set-in-webviewbuilder-was-all-white-screenshots)
  for why `goto`-after-build races the constellation.**
- `Page` owns the `WebView`, the `RenderingContext`, the `Servo` clone used to
  spin the event loop, and an `Rc<PageHandle>` that the `PageDelegate` mutates.

## Headed Mode

`crates/browsers-cli/src/headed.rs` builds a `WindowRenderingContext` from the
winit window, creates one `WebView` for a **hardcoded** `https://example.com`
(the CLI `open` URL argument is accepted but ignored), and drives
`servo.spin_event_loop()` on winit wake/redraw events. Per-frame:
`webview.paint()` then `rendering_context.present()`. Scroll and resize are
forwarded.

## Headless Mode

The headless path renders a URL to a PNG with **no window, no desktop session,
and no monitor** required. It reuses the same Servo instance setup as headed
mode, differing only in the rendering context (`SoftwareRenderingContext`
instead of `WindowRenderingContext`).

### Pipeline

```text
URL
  │
  v
HeadlessBrowser::render_url(url)
  ├── BrowserContext::new_page(url)     # WebViewBuilder::url(...) → build → resize viewport
  ├── Page::wait_for_load(timeout)      # spin servo loop until LoadStatus::Complete
  └── spin loop for wait_after_load_ms  # allow layout + paint frames to settle
  │
  v
Page::save_screenshot(path)             # webview.take_screenshot → RgbaImage → PNG on disk
```

`PageDelegate::notify_new_frame_ready` calls `webview.paint()` and bumps
`PageHandle.frames`. `PageDelegate::notify_load_status_changed` sets `loaded`
on `LoadStatus::Complete`.

### Public API

`crates/browsers-headless/src/lib.rs`:

```rust
pub struct HeadlessConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub load_timeout_ms: u64,      // default 30_000
    pub wait_after_load_ms: u64,   // default 3_000
}

pub struct HeadlessBrowser;

impl HeadlessBrowser {
    pub fn new(config: HeadlessConfig) -> Result<Self, HeadlessError>;
    pub fn render_url(&self, url: &str) -> Result<Page, HeadlessError>;
    pub fn screenshot_url(&self, url: &str, output_path: &str) -> Result<(), HeadlessError>;
    pub fn dump_html(&self, url: &str) -> Result<String, HeadlessError>;
    pub fn spin_event_loop(&self);
}
```

`Page` (from `crates/browsers-core/src/lib.rs`) adds:

- `goto(url)`, `wait_for_load(timeout_ms)`, `wait_for_render(timeout_ms)`
- `screenshot() -> RgbaImage`, `save_screenshot(path)`
- `evaluate_javascript(script, callback)`
- `surface_size()` (rendering-context size), `webview_size()` (webview render size)
- `surface_readback()` (diagnostics), `present()` (swaps buffer)
- `pixel_summary(img)` (stats helper: dimensions, distinct-color count, corner samples)

### Waiting Strategy

- `wait_for_load` spins `servo.spin_event_loop()` and
  `rendering_context.make_current()` until the delegate observes
  `LoadStatus::Complete`, or the deadline is hit.
- `screenshot_url` additionally spins for `wait_after_load_ms` (16ms sleep per
  iteration) so layout and paint pipe through. Timing-sensitive pages may need a
  larger value.
- The CLI passes `--timeout` → `load_timeout_ms`. Picture/network-heavy pages
  need a longer timeout (e.g. Wikipedia: 120 000 ms for a reliable render on a
  debug build).

### Pixel Capture — Three Ways

1. **`take_screenshot`** (what the PNG path uses). Reads the WebRender scene;
   returns an `RgbaImage` in the callback. **Always correct** regardless of
   surface swap state.
2. **`surface_readback`** — reads the software surface via `read_to_image`.
   **Reads the pending (pre-present) buffer**, so it must run before `present()`.
3. **`pixel_summary`** — debug helper printing dimensions, distinct-color count,
   and a few corner samples. Useful for eyeballing "is this a real page or a
   blank/white image".

### CLI Example

```bash
cargo run -p browsers-cli -- screenshot \
  https://en.wikipedia.org/wiki/Philosophy \
  --output /tmp/wiki.png \
  --width 1280 --height 720 \
  --timeout 120000
```

The URL is the positional argument. Flags: `--output` (default `screenshot.png`),
`--width` (1280), `--height` (720), `--timeout` (30 000) in ms. The URL must be
absolute (`file://`, `http://`, `https://`).

### Configuration Knobs

- `RUST_LOG=info` / `RUST_LOG=debug` to see `notify_load_status_changed`,
  frame-ready, take_screenshot size, and readback summaries.
- `device_scale_factor` lives on `BrowserConfig` (default 1.0).
- Viewport size is applied to both the `SoftwareRenderingContext` and the
  `WebView` via `webview.resize(...)`.

### Regression Test

`crates/browsers-headless/tests/headless_rendering.rs` exercises the full
pipeline in one test: viewport application, surface readback containing the
real document, and PNG output on disk. **Servo's opts are process-global, so
the whole pipeline runs in a single test.** Run with
`cargo test -p browsers-headless`.

## Server Mode

Implemented as the `browserverless serve` subcommand
(`crates/browsers-cli/src/serve.rs`), exposed through the `browsers_cli::serve`
lib module. A thin HTTP layer over `browsers_headless::HeadlessBrowser` — never
a custom HTML renderer; Servo renders.

### Endpoints

```text
GET  /api/v1/health            -> 200 {"status":"ok"}
GET  /api/v1/version           -> 200 <crate version>
POST /api/v1/scrape            -> 200 <full HTML of url> (text/html; charset=utf-8)
POST /api/v1/screenshot        -> 200 <PNG of url> (image/png)
```

`POST /api/v1/scrape` and `POST /api/v1/screenshot` take a JSON body
`{"url": "https://..."}` (parsed with a minimal std-only JSON string extractor;
surrogate pairs are not decoded). Errors are JSON `{"error":"..."}`: `400` —
missing/invalid URL or bad body; `404` — unknown path; `405` — non-POST on
scrape/screenshot; `504` — render timed out; `500` — other render failures.
Request bodies are capped at 64 KiB.

Both 200 scrape/screenshot responses carry metadata as headers:

```text
x-browserverless-url:          final URL after redirects (location.href)
x-browserverless-title:        page title; empty string when unavailable
x-browserverless-load-status:  "ok" or "partial" (load timed out; partial DOM)
x-browserverless-memory-kb:    process peak-RSS increase for the request
x-browserverless-duration-ms:  wall time for the request
```

Header values are ASCII-sanitized (non-ASCII/control characters become `?`).

### How It Honors the Hard Constraints

1. **One Servo instance per OS process** (`servo-config` opts are
   process-global anyway) → the server builds exactly one `HeadlessBrowser` and
   serializes every request through a single accept loop on its own dedicated
   thread.
2. **Multiple pages in one `BrowserContext` are not reliable yet** → each
   request creates one `Page` via `HeadlessBrowser` internals and drops it
   before the next request, in line with the one-page-at-a-time model. Known
   limitation: a long-lived single context is reused across requests (see
   [Servo Integration Notes §6](#6-multiple-pages-in-one-context-are-not-reliable));
   a per-request fork/heavier isolation is out of MVP scope.
3. **`SoftwareRenderingContext` needs a dedicated thread** → the whole server
   (Servo init + accept + render) runs on one worker thread, never an async
   executor.

### URL Policy

MVP is explicit trusted mode and the insecure behavior is documented: only
`http`/`https` schemes are accepted. Blocked: `file`, `data`, `javascript`,
custom schemes. Not yet implemented (hardening phase): blocking
localhost/private networks/link-local, domain allowlists.

### Limits and Logging

- Per-request load timeout (`--timeout`, default 30s). Max request body 64 KiB.
- Every request logs a structured line at `info`:
  `request_id method target status duration_ms`. Never logs cookies/auth/body.
- `--port <PORT>` overrides the port in `--bind` (default 8080). On startup a
  boxed banner shows the listening URL using the machine's external (LAN) IP
  when discoverable. Every request is written to stdout as
  `[YYYY-MM-DD HH:MM:SS] req=<n> <METHOD> <path> -> <status> (<duration>ms)`.
  When stdout is a terminal the banner and logs are ANSI-colored (status by
  class: green 2xx, cyan 3xx, yellow 4xx, red 5xx); piped output stays plain.

### Run

```bash
cargo run -p browsers-cli -- serve --bind 127.0.0.1:8080 --timeout 60000
curl -X POST 'http://127.0.0.1:8080/api/v1/scrape' \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/"}'
curl -X POST 'http://127.0.0.1:8080/api/v1/screenshot' \
  -H 'Content-Type: application/json' \
  -d '{"url":"https://example.com/"}' -o screenshot.png
```

### Tests

`crates/browsers-cli/tests/serve_api.rs` plus unit tests in `serve.rs`. **One
caveat:** Servo is process-global, so the test binary contains a **single**
integration test that exercises health/version/scrape/screenshot/scheme-
blocking/400/404/405 in one process.

### Out of MVP Scope (hardening)

Separate `server` crate + `browserverless-server` bin, concurrency limits,
request cancellation, JSON + full render (PNG) endpoints, SSRF hardening
(private-network blocking, allowlists), graceful shutdown, memory limits.

## MCP

Implemented as the `browserverless mcp serve` subcommand
(`crates/browsers-cli/src/mcp/`), exposed through the `browsers_cli::mcp` lib
module. A thin JSON-RPC layer over `browsers_headless::HeadlessBrowser` — the
same real Servo rendering path as Server Mode, never a custom renderer or a
stub.

### Transport

Newline-delimited JSON-RPC 2.0 over stdio. stdin is the request stream, stdout
carries replies as one line per frame, stderr carries diagnostics. Diagnostics
never go to stdout: a stray line on stdout would desynchronise the client.

`FrameReader` (`mcp/transport.rs`) caps a frame at 8 MiB. On overflow it
discards bytes until the next newline instead of parsing the tail, so an
oversized frame cannot be executed as a second command. Bytes read at EOF
without a trailing newline are a final frame, not a parse error, because a
client whose last frame lacks the newline is normal.

### Protocol

A request without an `id` is a notification: it is never answered, and the
notification check precedes the `jsonrpc` version check so a client cannot
receive an unmatched reply. `tools/call` `arguments` arrive as a JSON object
and are passed to handlers verbatim. A failure inside a tool — a bad URL, a
refused scheme, a render error — returns `isError: true` with a message the
model can act on; a protocol failure (unknown method, bad version, unparseable
frame) returns a JSON-RPC error. `tools/list` is sorted by name.

### One Browser per Process

`servo-config` `opts` are process-global and panic on second initialisation, so
building a `HeadlessBrowser` per tool call crashed the second call
(`Already initialized: Opts {...}`). `ServeRenderer` therefore holds
`Mutex<Option<HeadlessBrowser>>`, builds the browser on first use, and reuses
it; each call gets a fresh page within it. The lock is held for the whole
render, which keeps the thread-affine `BrowserContext` (it holds an `Rc`)
owned by one caller. Consequently `Renderer` does **not** require `Send`/`Sync`,
and the stdio server is single-threaded by design.

A per-call `timeout_ms` therefore cannot be expressed by rebuilding the
browser. `browsers-headless` gained `scrape_with_timeout`/
`screenshot_bytes_with_timeout` so the cached browser can honour a
per-request deadline; the existing `scrape` and `screenshot_bytes` delegate to
them with the configured default.

### Regression Tests

`crates/browsers-cli/tests/mcp.rs` drives the server over a stub renderer
(24 tests: initialize, ping, `tools/list`, `tools/call`, notifications, frame
errors, single-line output, `isError`, image blocks).
`crates/browsers-cli/tests/mcp_render.rs` exercises the real Servo renderer
against a local fixture and contains a **single** test, for the
one-Servo-per-process reason above: it renders twice in one session, which is
the regression that previously panicked.

## Surface Semantics (Important for Pixel Work)

- The software surface is **double-buffered** (surfman). `present()` swaps in a
  cleared buffer — **reading the surface after `present()` returns black**.
- Reading the surface _before_ `present()` returns the pending composed frame.
- `take_screenshot`, however, reads the WebRender scene directly and is not
  subject to the swap — the saved PNG is correct regardless of `present()`.

## Servo Integration Notes and Workarounds

Servo version: pinned `servo =0.8.0` in the workspace `Cargo.toml`
(`Cargo.lock` is authoritative). Registry path on this machine:

```text
~/.cargo/registry/src/index.crates.io-*/servo-0.8.0/
~/.cargo/registry/src/index.crates.io-*/servo-constellation-0.8.0/
```

Every Servo workaround is documented here and accompanied by a regression test.
This section was consolidated from the former `docs/servo-patches.md`.

### 1. Initial Navigation Must Be Set in `WebViewBuilder` (was: all-white screenshots)

**Symptom:** screenshots of any URL were uniformly white
(`surface distinct_colors=1`); the webview and surface were the right size —
the page simply never appeared.

**Root cause:** constructing a `WebView` starts the pipeline and creates the
top-level browsing context **asynchronously** in the constellation. Issuing
`webview.load(url)` immediately after `WebViewBuilder::build()` sent `LoadUrl`
before the browsing context was registered, and the constellation dropped it:

```text
WARN servo_constellation::constellation] PainterId: 1, TopLevelBrowsingContext(0,1):
     LoadUrl for unknown browsing context
```

Only the initial `about:blank` page (white — the default
`shell_background_color_rgba`) was ever rendered.

**Fix:** pass the URL at build time, which starts navigation inside `NewWebView`
handling (matching Servo's own `examples/winit_minimal.rs`):

```rust
// crates/browsers-core/src/lib.rs, BrowserContext::new_page
let webview: WebView = WebViewBuilder::new(&self.servo, self.rendering_context.clone())
    .url(parsed)                 // <-- initial URL here, not webview.load()
    .build();
```

**Regression test:** `crates/browsers-headless/tests/headless_rendering.rs` —
asserts the surface and the saved PNG contain the page background (red
`255,0,0,255`), not an empty page.

**Caveat:** `Page::goto` (later navigations) still calls `webview.load()`, but
pre-spins the event loop so the browsing context is registered before issuing
the load. Best-effort; later navigations under load are not yet exercised by
tests.

### 2. `(1, 1)` in "Ready to take screenshot of (1, 1)" Logs Is NOT a Size

The log prints a `PipelineId` as `(namespace_id, index)`. The WebView and
surface were verified at full viewport size throughout:

```rust
pub fn webview_size(&self) -> (f32, f32) { let s = self.webview.size(); ... } // == viewport
pub fn surface_size(&self) -> (u32, u32) { self.rendering_context.size2d() } // == viewport
```

### 3. Surfman Software Surface Is Double-Buffered

- Read back **before** `present()` → pending composed frame (distinct colors).
- Read back **after** `present()` → black cleared buffer; `present()` swapped
  in a fresh buffer. Do not use post-present readback for verification.
- `WebView::take_screenshot` reads the WebRender scene directly and is immune
  to the swap — use it for the PNG output path.

### 4. `webview.paint()` Is Async, One Message at a Time

`paint()` enqueues an async paint; winit's per-frame pattern is `paint()`
**then** `rendering_context.present()`. In headless mode, drive rendering by
repeatedly `servo.spin_event_loop()` + `make_current()` (see
`Page::wait_for_render`), not by assuming a synchronous compose.

### 5. One Servo Instance per Process (opts Panic)

`servo-config` `opts::set`/`preferences` are process-global and panic on second
initialization (`Already initialized: Opts {...}`). Because of this:

- The headless rendering test is a single test (second `HeadlessBrowser::new`
  would panic).
- Server/concurrency code must keep to one Servo per process or fork processes
  (see [Server Mode](#server-mode)).
- The MCP server builds one browser and reuses it (see [MCP](#mcp)).

### 6. Multiple Pages in One Context Are Not Reliable

A second `new_page()` in the same `BrowserContext` captured the first page's
content (green fixture rendered as red in the test). Root cause not yet fully
traced; likely related to WR document/pipeline reuse across WebViews in one
process. Until diagnosed: one page per `BrowserContext`, one context per
process (the current CLI/headless layout).

### 7. Known Harmless Log Noise

- `WARN ... UNSUPPORTED (log once): POSSIBLE ISSUE: unit 1 GL_TEXTURE_INDEX_2D
  is unloadable ...` — surfman/GL noise on macOS; screenshots render correctly.
- `webrender` `Cropping texture upload Box2D((0, 0), (0, 1))` debug asserts —
  harmless crop of near-empty rects; not the cause of any observed bug.

### 8. Load-Status ↔ Timeout Mapping

- `load_timeout_ms` waits for `LoadStatus::Complete` via the delegate.
  `hard_fail: true` (default opts) means load failures surface as
  timeouts/failures rather than a rendered error page.
- The `HeadlessConfig`/`HeadlessError::Timeout` boundary is where navigation
  failures and timeouts are collapsed for the CLI.

## Compatibility

These are **verified** rendering results from the current headless pipeline
(debug build, macOS arm64), status as of 2026-08-28.

### Verified Rendering Targets

| Target                                                                   | Result                                                                                                                                                           |
| ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/browsers-cli/tests/fixtures/rendering/mvp.html` (golden fixture) | Border bbox `x[50..393] y[50..193]` (344×144 = 300 + 2×20 padding + 2×2 border), interior `rgb(211,211,211)`, white wallpaper. Pixel-exact.                      |
| `file:///tmp/<red-or-green>.html` (solid-color bodies)                   | Exact background color in surface readback **and** saved PNG.                                                                                                    |
| `https://example.com/`                                                   | `#eeeeee` background, dark text, 134 distinct sampled colors — real page.                                                                                        |
| `https://en.wikipedia.org/wiki/Philosophy`                               | White article bg, `#f8f9fa` sidebar, `(32,33,34)` text, `#3366cc` link blue, ~8199 distinct sampled colors. No TLS/cert errors.                                  |
| `https://www.instagram.com/p/Db-W3jykdhs/`                               | Renders the logged-out post page (dark `#54575A` backdrop, white login card, Instagram blue `#4a5df9`, ~3400 distinct colors) via the surface-readback fallback. |

Note: `crates/browsers-cli/tests/fixtures/rendering/mvp.html` requires
`body { margin: 0 }` for the golden geometry; an 8px body margin (browser
default) shifts the box and breaks the pixel checks.

### Known Limitations (pre-1.0 Servo embedding)

- **One Servo instance per process.** A second `ServoBuilder::...build()`
  panics (`Already initialized`).
- **One page per `BrowserContext`.** A second page in the same process showed
  the first page's content. Single-page-per-process is the supported topology
  today.
- **Initial navigation must come from `WebViewBuilder::url()`.** `load()` right
  after `build()` races the constellation and is dropped (white screenshot).
  See [Servo Integration Notes §1](#1-initial-navigation-must-be-set-in-webviewbuilder-was-all-white-screenshots).
- **No network-idle condition.** Waiting is `LoadStatus::Complete` + fixed
  `wait_after_load_ms`. JS-driven or layout-late pages may need a larger wait.
- **Screenshot of never-idle pages is racy.** `take_screenshot` only fires its
  callback once the page has **no pending frames**; continuously-animating
  pages (e.g. Instagram) never satisfy that, so it times out by design. The
  fallback reads the surfman surface first (before the take_screenshot request,
  which presents/rotates buffers) and polls for a frame with real content, but
  that read races the compositor's present cycle and can still capture an early
  all-white frame. Output is a valid PNG either way; treat such captures as
  best-effort.
- **Headed `open` ignores its URL argument** and hardcodes `https://example.com`.
- **`hard_fail: true` default.** Load failures time out rather than render an
  error page.

### Unsupported (non-goals)

DRM, WebRTC, WebGPU, PDF output, printing, PWAs, full devtools, accessibility
UI, browser profile management — all explicitly deferred. Media (video/audio)
requires feature-flag and license review; it is not enabled.

### Performance Observations (debug build)

- Lightweight local/fixed pages: ~1 s end-to-end.
- Heavy pages (Wikipedia): use `--timeout 120000`; rendering completes but a
  debug build is slow. Measure with release builds before optimizing.
- Binary size: measure release builds only (`cargo build --release`).

### What to Re-Verify After Any Change

1. `cargo test -p browsers-headless` (the rendering regression test).
2. The mvp golden geometry:
   `screenshot file://$(pwd)/crates/browsers-cli/tests/fixtures/rendering/mvp.html --output /tmp/mvp.png --width 800 --height 600 --timeout 60000`.
3. One real URL (e.g. `example.com`) to catch networking/log-noise regressions.

## Key Invariants

1. **Servo is once-per-process.** `servo-config` `opts`/`prefs` initialize once
   globally and panic on a second `ServoBuilder::...build()` in the same
   process.
2. **Multiple pages in one `BrowserContext` are not reliable yet.** A second
   `new_page()` in the same process captured the first page's content (test
   evidence). Headless and CLI currently create exactly one page per process.
   See [Servo Integration Notes §6](#6-multiple-pages-in-one-context-are-not-reliable).
3. `webview.paint()` only sends one async message to the paint thread; there is
   no per-frame present/poll in the library API for software contexts. Loop via
   `spin_event_loop()`.
4. The default `shell_background_color_rgba` is white. A page that is never
   navigated (dropped load) renders as a white image, not black.
