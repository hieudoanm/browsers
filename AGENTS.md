# Lightweight Servo Browser — AI Agent Development Guide

## 0. Mission

This repository builds a **lightweight browser application powered by Servo**.

The project is intentionally **Servo-only as its browser engine**:

- Do not build a browser engine from scratch.
- Do not embed Chromium, CEF, Gecko, WebKit, WebKitGTK, WebView2, Electron, or Tauri WebView as an alternative engine.
- Use Servo's embedding API and Servo's own crates where appropriate.
- Build the browser product, shell, CLI, server interface, configuration, storage policy, and user experience around Servo.

The defining product requirement is:

> **One Rust codebase, one browser engine, two execution modes: headed and headless.**

### Headed mode

```text
mybrowser https://example.com
```

Opens an interactive desktop browser.

### Headless mode

```text
mybrowser --headless https://example.com
```

Runs without an interactive browser window and can be used for:

- screenshots
- rendering
- automation
- testing
- extraction
- server/API workloads
- future PDF output

### Server mode

```text
mybrowser-server --bind 0.0.0.0:8080
```

The server uses the same Servo browser core as the desktop application.

---

# 1. Critical Architectural Decision

## Servo is the browser engine

Do not recreate:

- HTML parsing
- CSS parsing
- DOM implementation
- JavaScript engine
- layout engine
- browser rendering engine
- Web platform implementation

Servo already provides these capabilities.

The application should primarily be an **embedder/browser shell**.

Conceptually:

```text
                         Browser Application
                                |
                     +----------+----------+
                     |                     |
                 Headed Shell          Headless Shell
                     |                     |
                     +----------+----------+
                                |
                         Browser Runtime
                                |
                              Servo
                                |
             +------------------+------------------+
             |                  |                  |
            DOM                JS               Layout
             |                  |                  |
             +------------------+------------------+
                                |
                           WebRender
                                |
                    +-----------+-----------+
                    |                       |
              Window target          Offscreen target
                    |                       |
                  Desktop                Image/API
```

The application MUST NOT maintain two independent rendering implementations.

---

# 2. Important Servo Reality

Servo is an actively evolving embeddable browser engine.

At the time this document was written, Servo provides a `servo` crate intended for embedding, but its embedding API is still pre-1.0 and can change between releases.

Therefore:

1. Pin the Servo version.
2. Keep Servo integration isolated.
3. Do not scatter Servo-specific types throughout the entire application.
4. Create a small internal adapter layer around Servo.
5. Upgrade Servo deliberately.
6. Run the complete browser test suite after every Servo upgrade.

Prefer the current stable/LTS Servo release when appropriate rather than tracking an arbitrary Git revision.

The project should not assume Servo's API is permanently stable.

---

# 3. Product Scope

## 3.1 MVP

The MVP is a browser shell that can:

### Headed

- open a URL
- render HTML/CSS/JS using Servo
- display the page
- navigate links
- go back
- go forward
- reload
- scroll
- resize viewport
- accept keyboard/mouse input
- display basic navigation errors

### Headless

- load a URL
- wait for page load
- wait for a configurable condition or timeout
- render to an offscreen target
- save screenshots
- return rendering results programmatically
- run without an interactive browser window

### Server

- accept a rendering request
- create an isolated browser/page context
- navigate
- wait
- render
- return a result
- clean up resources

Example:

```text
POST /render
{
  "url": "https://example.com",
  "viewport": {
    "width": 1280,
    "height": 720
  }
}
```

The exact HTTP API may evolve.

---

# 4. Non-Goals for MVP

Do NOT prioritize:

- extensions
- WebExtensions
- DRM
- Widevine
- WebRTC
- WebGPU
- browser synchronization
- password manager
- account system
- cloud sync
- ad blocker
- built-in VPN
- full developer tools
- browser history UI
- bookmarks UI
- multiple profiles
- private browsing UX
- advanced downloads manager
- PDF generation
- printing
- complete accessibility UI
- complete browser settings
- complete browser compatibility

These may be added later.

Do not allow them to delay the core headed/headless pipeline.

---

# 5. Technology Policy

## Required

- Rust
- Cargo
- Servo
- Servo's WebView/embedder APIs
- Servo/WebRender rendering stack
- A platform window/event integration appropriate for the supported target
- CLI argument parsing
- HTTP server library for server mode

## Strong preference

Use Servo's existing abstractions instead of duplicating them.

Use small Rust dependencies for:

- CLI
- HTTP
- serialization
- configuration
- logging
- testing

## Forbidden as browser engines

Do not introduce:

- Chromium
- CEF
- Electron
- Gecko
- Firefox embedding
- WebKit
- WebKitGTK
- WKWebView
- WebView2
- Tauri WebView
- Playwright as the rendering engine
- Puppeteer as the rendering engine

Playwright/Puppeteer may be used only for external compatibility experiments if explicitly approved; they must never become runtime dependencies of the browser.

---

# 6. Repository Architecture

Prefer a small workspace.

Start with:

```text
.
├── AGENTS.md
├── Cargo.toml
├── Cargo.lock
├── README.md
│
├── crates/
│   ├── browser/
│   ├── shell/
│   ├── headless/
│   ├── server/
│   └── cli/
│
├── tests/
│   ├── fixtures/
│   ├── rendering/
│   └── integration/
│
├── docs/
│   ├── architecture.md
│   ├── compatibility.md
│   └── development.md
│
└── tasks/
    ├── 000-bootstrap.md
    ├── 001-servo-embedding.md
    ├── 002-headed-navigation.md
    ├── 003-headless-rendering.md
    └── ...
```

Do NOT create a crate for every small concept.

Split a crate only when it has:

- independent ownership,
- a meaningful public API,
- separate dependencies,
- separate tests,
- or a clear architectural boundary.

---

# 7. Recommended Responsibilities

## `browser`

Own application-level browser concepts:

- Browser
- BrowserContext
- Page
- Navigation
- Viewport
- Browser configuration
- lifecycle
- Servo integration boundary

It should NOT know about:

- terminal UI
- HTTP server routing
- desktop toolbar widgets

---

## `shell`

Own the interactive desktop experience:

- window creation
- input events
- keyboard shortcuts
- navigation controls
- URL/address bar
- viewport management
- page display

It should call the browser core.

---

## `headless`

Own:

- offscreen execution
- rendering requests
- screenshot capture
- deterministic execution
- wait conditions
- resource limits

It should not contain a second browser engine.

---

## `server`

Own:

- HTTP API
- request validation
- authentication if eventually required
- concurrency
- request cancellation
- browser context lifecycle
- resource limits

The server must call `browser`/`headless`.

---

## `cli`

Own:

- command parsing
- configuration loading
- process startup
- exit codes
- human-readable errors

The CLI should not contain browser logic.

---

# 8. Core Domain Model

Use concepts similar to:

```text
Browser
  |
  +-- BrowserContext
        |
        +-- Page
              |
              +-- WebView / Servo integration
```

Possible public API:

```rust
Browser::new(config)
Browser::new_context(options)
BrowserContext::new_page()
Page::goto(url)
Page::reload()
Page::go_back()
Page::go_forward()
Page::screenshot(options)
Page::evaluate(script)
Page::close()
```

The exact API is not fixed.

Before inventing APIs, inspect the current Servo embedding API and adapt to it.

Do not create a large abstraction layer that merely renames every Servo method.

---

# 9. Servo Integration Layer

Create one clear integration boundary.

For example:

```text
crates/browser/src/servo/
├── mod.rs
├── browser.rs
├── webview.rs
├── delegate.rs
├── events.rs
├── rendering.rs
└── input.rs
```

The rest of the application should depend primarily on project-level types.

Avoid:

```rust
pub fn foo(webview: servo::WebView)
```

throughout unrelated modules.

Prefer:

```rust
pub fn foo(page: &Page)
```

and keep the Servo object inside the browser implementation where possible.

---

# 10. Headed Mode

Headed mode is an application shell around Servo.

Pipeline:

```text
OS Event
   |
Window/Event Loop
   |
Browser Shell
   |
Page
   |
Servo WebView
   |
Servo Renderer
   |
WebRender
   |
Window Rendering Context
```

The shell is responsible for:

- creating the window
- forwarding input
- forwarding resize events
- presenting rendered content
- handling application lifecycle

The shell must NOT implement HTML/CSS/JS behavior.

---

# 11. Headless Mode

Headless mode is a first-class product feature.

Do not implement it as:

```text
headed browser
+ hide window
```

unless Servo's current embedding API explicitly makes that the correct rendering configuration.

Instead, use Servo's rendering-context abstractions to create a genuine offscreen/headless path where supported.

The desired architecture is:

```text
Headless Request
      |
BrowserContext
      |
Page
      |
Servo WebView
      |
Rendering Context
      |
Offscreen framebuffer/image
      |
Screenshot / API response
```

Headless mode must not require:

- a physical monitor
- an interactive desktop session
- a visible browser window

If a platform requires a graphics backend for offscreen rendering, document that requirement rather than pretending the system is fully CPU-only.

---

# 12. Server Mode

Server mode should be a thin layer over headless mode.

```text
HTTP
 |
Request
 |
Validation
 |
BrowserContext
 |
Page
 |
Servo
 |
Headless Rendering
 |
Response
```

Never do:

```text
HTTP
 |
custom HTML renderer
```

The server MUST use Servo.

---

# 13. Server Isolation

Every request must have explicit ownership of browser state.

At minimum:

```text
Request
  |
  +-- BrowserContext
        |
        +-- Page
```

Do not accidentally share:

- DOM
- cookies
- local storage
- session state
- JavaScript globals
- navigation state

between unrelated requests.

Decide explicitly whether contexts are:

- per request,
- per session,
- reusable pools.

The default MVP should favor isolation and correctness over maximum throughput.

---

# 14. Concurrency

Do not assume Servo can safely be used from arbitrary threads.

Follow Servo's threading and event-loop requirements.

Before implementing concurrent server rendering:

1. inspect the current Servo embedding API,
2. identify thread-affinity constraints,
3. document them,
4. build a small single-request implementation,
5. add concurrency only after correctness is proven.

A safe initial architecture is:

```text
HTTP server
     |
request queue
     |
browser worker(s)
     |
Servo
```

Do not create one OS thread per request without measuring.

---

# 15. Navigation

Navigation should eventually support:

- HTTP
- HTTPS
- redirects
- relative URLs
- fragment navigation
- link clicks
- history
- reload
- navigation cancellation
- navigation errors

Use Servo's navigation facilities where available.

Do not write a second URL or navigation implementation unless necessary.

---

# 16. Waiting and Lifecycle

Headless rendering needs deterministic lifecycle control.

Define explicit concepts such as:

```text
NavigationStarted
NavigationCommitted
DOMReady
LoadEvent
NetworkIdle (if implementable)
RenderReady
Timeout
NavigationFailed
```

Do not rely only on arbitrary sleeps:

```rust
sleep(Duration::from_secs(3));
```

Prefer:

```text
wait for load
wait for network condition
wait for script condition
wait for rendering readiness
```

A timeout may still be provided as a safety limit.

---

# 17. Screenshot API

The first headless output should be PNG.

Example:

```bash
mybrowser \
  --headless \
  --viewport 1280x720 \
  --screenshot screenshot.png \
  https://example.com
```

API concept:

```rust
Page::screenshot(ScreenshotOptions {
    width: 1280,
    height: 720,
    full_page: false,
    ..Default::default()
})
```

Do not implement image processing in the browser core unless necessary.

---

# 18. Full-Page Screenshots

Treat full-page screenshots separately from viewport screenshots.

Viewport:

```text
+----------------------+
|                      |
|      viewport        |
|                      |
+----------------------+
```

Full page:

```text
+----------------------+
|                      |
|                      |
|                      |
|                      |
|                      |
|                      |
+----------------------+
```

Do not fake full-page screenshots by simply enlarging the viewport until the page fits.

Use layout/content dimensions where Servo exposes the necessary information.

---

# 19. JavaScript

Servo's JavaScript runtime should be used.

Do not introduce a second JavaScript engine.

The application should expose only browser APIs that Servo supports.

For custom application functionality, use explicit bindings rather than modifying the web platform unnecessarily.

Never expose host capabilities such as:

- arbitrary filesystem access
- shell execution
- environment variables
- arbitrary process execution

to untrusted webpage JavaScript.

---

# 20. Security Model

Treat all remote webpage content as untrusted.

Do not disable browser security merely to simplify development.

Never add:

```text
allow-all filesystem
allow-all shell
allow-all native calls
```

to make a test pass.

When Servo has a known limitation, document it.

For server mode, additionally consider:

- request timeouts
- maximum page size
- maximum screenshot dimensions
- maximum concurrent pages
- memory limits
- navigation restrictions
- private-network access
- DNS rebinding
- SSRF
- file:// access
- localhost access
- process lifetime

Server rendering is potentially an SSRF service. Treat URL input as security-sensitive.

---

# 21. Server URL Policy

A production server MUST NOT blindly fetch arbitrary URLs supplied by clients.

Eventually implement configurable policies such as:

```text
allowed schemes:
  http
  https

blocked:
  file
  data
  javascript
  custom schemes

optional:
  block localhost
  block private networks
  block link-local addresses
  allowlist domains
```

The exact policy belongs in server configuration.

MVP can start with an explicit trusted mode, but the insecure behavior must be documented.

---

# 22. Browser State

Separate:

```text
Browser configuration
BrowserContext state
Page state
Application UI state
```

Do not put everything in a global singleton.

Avoid:

```rust
static mut GLOBAL_BROWSER
```

Prefer explicit ownership.

---

# 23. GUI Design

The first GUI should be intentionally minimal.

```text
+------------------------------------------------+
| ←  →  ↻   [ https://example.com             ] |
+------------------------------------------------+
|                                                |
|                                                |
|                 Servo Page                     |
|                                                |
|                                                |
+------------------------------------------------+
```

Implement:

1. address bar
2. back
3. forward
4. reload
5. page viewport
6. basic error state

Do not build a sophisticated browser UI before the runtime is reliable.

---

# 24. Multiple Tabs

Tabs are NOT an MVP requirement.

When implemented:

```text
Browser
 |
 +-- Tab
 |     |
 |     +-- Page
 |
 +-- Tab
       |
       +-- Page
```

Do not model tabs as multiple browser processes unless required.

Use Servo's multi-WebView model appropriately.

---

# 25. Profiles and Storage

Eventually support:

```text
Browser
 |
 +-- Profile
       |
       +-- cookies
       +-- local storage
       +-- cache
       +-- preferences
```

Do not make global mutable storage the default.

Server mode should use isolated temporary or explicitly configured storage.

---

# 26. Testing Strategy

Testing is a first-class engineering requirement.

Use:

1. unit tests
2. integration tests
3. rendering tests
4. headless tests
5. browser lifecycle tests
6. server API tests
7. regression tests
8. Web Platform Tests where practical

---

# 27. Rendering Golden Tests

Store fixtures as:

```text
crates/browsers-cli/tests/fixtures/rendering/
├── basic/
│   ├── page.html
│   └── expected.png
│
├── css/
│   ├── box-model.html
│   └── expected.png
│
├── text/
│   ├── wrapping.html
│   └── expected.png
│
└── images/
    ├── image.html
    └── expected.png
```

Tests should verify:

- page loads
- rendering completes
- screenshot exists
- dimensions are correct
- pixels are acceptably close to the baseline

Use a defined image-difference threshold rather than exact equality when platform rendering makes exact equality unrealistic.

---

# 28. Deterministic Headless Rendering

The headless test environment should control:

- viewport size
- device scale factor
- fonts
- timezone
- locale
- random seeds where possible
- current time where possible
- network dependencies
- animations
- external resource availability

Do not use live websites as the primary rendering test suite.

Use local fixtures.

---

# 29. Web Platform Tests

Do not attempt to implement a custom browser compatibility suite.

Leverage:

- Servo's existing tests
- Web Platform Tests
- HTML parsing tests
- CSS tests
- relevant upstream Servo tests

When a Servo feature already works, do not reimplement it.

When the application wrapper causes a failure, add an application-level regression test.

---

# 30. Test Fixtures

Prefer tiny fixtures.

Good:

```html
<div class="box">Hello</div>
```

Bad:

```text
entire production website saved into repository
```

Every regression should have the smallest useful reproduction.

---

# 31. Offline Tests

The majority of tests should work without the public internet.

Use:

- local HTML
- local CSS
- local JS
- local images
- local fonts
- local HTTP test servers

Do not make CI depend on example.com.

---

# 32. Server Tests

Test:

```text
POST /render
GET /health
GET /version
```

At minimum.

Test:

- valid URL
- invalid URL
- timeout
- unreachable host
- HTTP error
- redirect
- large page
- JavaScript page
- concurrent requests
- request cancellation
- malformed request
- oversized viewport
- resource cleanup

---

# 33. Error Handling

A webpage must not crash the browser.

Convert failures into structured errors:

```rust
BrowserError::Navigation(...)
BrowserError::Timeout(...)
BrowserError::Rendering(...)
BrowserError::InvalidUrl(...)
BrowserError::Unsupported(...)
BrowserError::Resource(...)
```

Do not use `unwrap()` or `expect()` in request-controlled or webpage-controlled paths.

Panics should represent actual programmer invariants, not normal browser failures.

---

# 34. Logging

Use structured logging.

Useful fields:

```text
request_id
context_id
page_id
url
navigation_id
duration_ms
status
error
```

Avoid logging:

- cookies
- authorization headers
- credentials
- page secrets
- arbitrary POST bodies

unless explicitly enabled for debugging.

---

# 35. Performance Targets

Do not optimize blindly.

Measure:

### Startup

```text
process startup
Servo initialization
first page
```

### Navigation

```text
DNS
connect
request
response
parse
layout
paint
first render
```

### Rendering

```text
layout time
paint time
WebRender time
screenshot time
```

### Memory

```text
browser baseline
one page
multiple pages
server concurrency
```

Establish baselines before optimization.

---

# 36. Resource Limits

Headless/server rendering must have explicit limits.

Examples:

```text
max navigation time
max page size
max screenshot width
max screenshot height
max concurrent pages
max request body
max redirects
max JavaScript execution time where supported
```

Never allow unbounded server resources.

---

# 37. CLI

The CLI should eventually support:

```bash
# Headed
mybrowser https://example.com

# Headless
mybrowser --headless https://example.com

# Screenshot
mybrowser --headless \
  --screenshot output.png \
  https://example.com

# Viewport
mybrowser --headless \
  --viewport 1280x720 \
  --screenshot output.png \
  https://example.com

# Script
mybrowser --headless \
  --script script.js \
  https://example.com

# Dump DOM if supported by Servo/application API
mybrowser --dump-dom https://example.com

# Version
mybrowser --version
```

The CLI must call the same browser runtime as server mode.

---

# 38. Server CLI

Example:

```bash
mybrowser-server \
  --bind 127.0.0.1:8080 \
  --workers 2 \
  --max-concurrency 8
```

Configuration should eventually support:

```text
config.toml
environment variables
CLI flags
```

Define precedence clearly.

---

# 39. API Versioning

The HTTP API should be versioned from the beginning:

```text
/api/v1/render
```

Do not expose internal Servo types through JSON.

Use stable application-level request/response schemas.

---

# 40. Documentation

Maintain:

```text
README.md
CONTRIBUTING.md
packages/landing/md/docs/**/*.md   # published docs site (architecture, packaging, roadmap, ...)
```

README should answer:

1. What is this?
2. Why Servo?
3. How do I build it?
4. How do I run headed mode?
5. How do I run headless mode?
6. How do I run server mode?
7. What platforms are supported?
8. What web features are supported?
9. What is intentionally unsupported?

---

# 41. Servo Version Management

Pin Servo deliberately.

Record:

```text
Servo version
Rust MSRV
target platforms
known incompatibilities
```

When upgrading Servo:

1. update dependency
2. update lockfile
3. compile all targets
4. run unit tests
5. run headless tests
6. run rendering tests
7. run server tests
8. inspect performance
9. inspect binary size
10. update compatibility notes

Never upgrade Servo as an incidental part of an unrelated task.

---

# 42. Agent Development Workflow

Every AI agent MUST follow:

```text
READ
  ↓
UNDERSTAND
  ↓
INSPECT
  ↓
RESEARCH
  ↓
PLAN
  ↓
IMPLEMENT
  ↓
TEST
  ↓
REVIEW
  ↓
DOCUMENT
  ↓
VERIFY
```

## READ

Read:

- AGENTS.md
- relevant task file
- README
- architecture docs

## UNDERSTAND

Identify:

- current architecture
- existing APIs
- Servo integration point
- affected tests

## INSPECT

Search the repository before creating new code.

Never assume a feature does not already exist.

## RESEARCH

For Servo behavior:

1. inspect current Servo source/docs
2. inspect existing Servo examples
3. inspect current API
4. check relevant standards
5. inspect existing tests

Do not rely on stale memory of Servo APIs.

## PLAN

Before modifying multiple files, write a concise implementation plan.

## IMPLEMENT

Make the smallest coherent change.

## TEST

Run the smallest relevant tests first, then the broader suite.

## REVIEW

Ask:

- Did I duplicate Servo functionality?
- Did I create a second rendering path?
- Did I introduce a platform-specific assumption?
- Did I weaken security?
- Did I add a regression test?
- Did I unnecessarily increase dependencies?

## DOCUMENT

Update docs when architecture or behavior changes.

## VERIFY

Never claim a task is complete without actual verification.

---

# 43. AI Agent Rules

AI agents MUST:

1. Read AGENTS.md first.
2. Inspect the current code before changing it.
3. Search before creating abstractions.
4. Prefer Servo APIs over reimplementations.
5. Keep headed and headless paths on the same browser core.
6. Add tests for behavior changes.
7. Add regression tests for bugs.
8. Never silently remove failing tests.
9. Never weaken assertions just to pass CI.
10. Never add website-specific hacks.
11. Never add another browser engine.
12. Never expose host privileges to webpage JavaScript.
13. Never assume server input is trusted.
14. Run formatting.
15. Run clippy.
16. Run relevant tests.
17. Run rendering tests for rendering changes.
18. Run server tests for server changes.
19. Document blockers instead of pretending work is complete.
20. Keep commits focused.

---

# 44. Servo-Specific Agent Rules

Before writing code that interacts with Servo:

1. Inspect the exact Servo version in Cargo.lock.
2. Inspect current Servo API documentation.
3. Search Servo source for existing patterns.
4. Prefer the public embedding API.
5. Avoid depending on Servo internals unless unavoidable.
6. Isolate unavoidable internal dependencies.
7. Document any Servo-specific workaround.
8. Add a regression test.

Do not write code based on an old Servo API from memory.

---

# 45. Task System

Complex development should be broken into Markdown tasks.

```text
tasks/
├── 000-bootstrap.md
├── 001-servo-embedding.md
├── 002-headed-shell.md
├── 003-navigation.md
├── 004-input.md
├── 005-headless-context.md
├── 006-screenshot.md
├── 007-cli.md
├── 008-server.md
├── 009-server-isolation.md
├── 010-rendering-tests.md
├── 011-storage.md
├── 012-tabs.md
└── ...
```

Task states:

```text
pending
researching
planned
implementing
testing
reviewing
blocked
completed
```

Task template:

```markdown
# Task: <name>

## Status

pending

## Goal

Describe the outcome.

## Context

Why this task exists.

## Requirements

- ...

## Non-goals

- ...

## Servo APIs / References

- ...

## Implementation Plan

1. ...
2. ...
3. ...

## Tests

- ...

## Acceptance Criteria

- [ ] ...
- [ ] ...

## Risks

- ...

## Notes

...
```

---

# 46. Recommended Development Phases

The following is the default roadmap.

Agents may adjust it when repository evidence requires a different order.

---

## Phase 0 — Repository Bootstrap

Goal:

```text
Cargo workspace
+
CI
+
formatting
+
linting
+
basic executable
```

Tasks:

- initialize workspace
- configure Rust edition
- configure rustfmt
- configure clippy
- configure CI
- pin Servo
- build minimal binary

Acceptance:

```bash
cargo check
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Phase 1 — Servo Smoke Test

Goal:

> Prove that the application can embed Servo.

Tasks:

- initialize Servo
- create a WebView
- load local HTML
- create the required event loop/rendering context
- verify a page renders
- establish application-level Servo adapter

Acceptance:

```text
local HTML loads
Servo initializes
application exits cleanly
no custom browser engine exists
```

---

## Phase 2 — Headed Browser

Goal:

> Display a real webpage in a desktop window.

Tasks:

- create window
- create Servo WebView
- render page
- handle resize
- forward keyboard input
- forward pointer input
- scroll
- navigation

Acceptance:

```bash
mybrowser https://example.com
```

works on the first supported desktop target.

---

## Phase 3 — Navigation Shell

Implement:

- address bar
- back
- forward
- reload
- URL parsing
- navigation state
- load errors
- keyboard shortcuts

Keep UI minimal.

---

## Phase 4 — Headless Rendering

Goal:

> Run Servo without an interactive browser window.

Tasks:

- create headless rendering context
- create WebView
- navigate
- wait for load
- capture pixels
- write PNG
- clean up

Acceptance:

```bash
mybrowser \
  --headless \
  --screenshot output.png \
  https://example.com
```

works without a visible browser window.

Also test on CI/server environments appropriate to the target platform.

---

## Phase 5 — Deterministic Rendering

Implement:

- fixed viewport
- fixed scale factor
- controlled fonts
- deterministic fixtures
- local test server
- screenshot comparison
- animation control where possible

Acceptance:

Repeated renders of the same fixture produce stable results within the defined image-diff threshold.

---

## Phase 6 — CLI

Implement:

```text
--headless
--viewport
--screenshot
--timeout
--user-agent
--script
--version
```

Only add flags when backed by actual functionality.

---

## Phase 7 — Server

Goal:

> Run the browser as an HTTP rendering service.

Implement:

```text
GET  /api/v1/health
GET  /api/v1/version
POST /api/v1/render
```

Initial render request:

```json
{
  "url": "https://example.com",
  "viewport": {
    "width": 1280,
    "height": 720
  }
}
```

Acceptance:

- server starts without GUI
- request creates an isolated page
- Servo renders page
- screenshot is returned
- resources are cleaned up
- timeout works

---

## Phase 8 — Server Hardening

Implement:

- concurrency limits
- request timeouts
- URL validation
- SSRF protection
- response size limits
- screenshot size limits
- structured logging
- graceful shutdown
- cancellation
- memory/resource cleanup

---

## Phase 9 — Browser State

Implement:

- cookies
- storage
- cache policy
- profiles
- persistent configuration

Only implement features that Servo's current APIs support cleanly.

---

## Phase 10 — Multiple Pages / Tabs

Implement:

- tab model
- multiple WebViews
- active tab
- tab lifecycle
- tab navigation
- tab close

---

## Phase 11 — Browser Product Features

Only after the runtime is stable:

- history
- bookmarks
- downloads
- settings
- keyboard command palette
- session restoration

---

## Phase 12 — Compatibility

Measure real-world compatibility.

Use:

- Web Platform Tests
- Servo tests
- selected real-world websites
- regression fixtures

Maintain:

```text
packages/landing/md/docs/project/architecture.md
```

with explicit known limitations (see §Compatibility).

---

# 47. MVP Acceptance Test

The first major milestone is:

### Input

```html
<!doctype html>
<html>
  <head>
    <style>
      body {
        margin: 0;
        font-family: sans-serif;
        background: white;
      }

      .box {
        width: 300px;
        height: 100px;
        margin: 50px;
        padding: 20px;
        background: lightgray;
      }
    </style>
  </head>
  <body>
    <div class="box">Hello Browser</div>
  </body>
</html>
```

### Headed

```bash
mybrowser tests/fixtures/basic.html
```

Must display the page.

### Headless

```bash
mybrowser \
  --headless \
  --viewport 800x600 \
  --screenshot /tmp/basic.png \
  tests/fixtures/basic.html
```

Must produce a valid PNG.

### Server

```bash
mybrowser-server --bind 127.0.0.1:8080
```

Then:

```text
POST /api/v1/render
```

must render the same HTML using the same Servo-based browser runtime.

This milestone proves the project's central architecture.

---

# 48. Definition of Done

A feature is complete only when:

- implementation exists
- API ownership is clear
- Servo integration is correct
- tests exist
- relevant tests pass
- clippy passes
- formatting passes
- rendering tests pass when applicable
- server tests pass when applicable
- no security regression exists
- documentation is updated when necessary
- acceptance criteria are verified

"It works on my machine" is not sufficient.

---

# 49. Git Discipline

Use focused commits:

```text
feat(servo): initialize embedded browser
feat(shell): add headed navigation
feat(headless): add offscreen screenshot
feat(cli): add screenshot command
feat(server): add render endpoint
test(rendering): add basic page fixture
fix(headless): wait for rendering completion
```

Do not combine unrelated refactors with feature work.

Do not rewrite working code merely because you prefer a different style.

---

# 50. Performance and Binary Size

The browser is intended to be lightweight.

Track:

- release binary size
- installed application size
- startup time
- first render time
- memory usage
- headless throughput

Measure before optimizing.

Use release builds for meaningful binary-size measurements.

Consider:

- LTO
- symbol stripping
- dependency feature minimization
- release profile tuning

Do not remove useful Servo functionality solely to reduce binary size before measuring its impact.

---

# 51. Platform Strategy

Start with **one primary desktop/server platform**.

Recommended initial target:

```text
Linux x86_64
```

because it simplifies server and desktop experimentation.

Then expand to:

```text
macOS arm64
Windows x86_64
Linux arm64
```

Do not claim cross-platform support until headed and headless paths have been tested on the target.

The application should keep platform-specific code behind small modules.

---

# 52. Platform Abstraction

Prefer:

```text
platform/
├── window/
├── input/
├── graphics/
└── clipboard/
```

rather than spreading:

```rust
#[cfg(target_os = "...")]
```

through every module.

Platform-specific code should live near the platform boundary.

---

# 53. Accessibility

Do not ignore accessibility permanently.

When the core browser is stable, integrate Servo's accessibility facilities into the host application.

The embedder should eventually expose the Servo accessibility tree through the appropriate platform adapter.

Do not invent a parallel DOM accessibility implementation.

---

# 54. Media

Media support is not an MVP requirement.

When adding media:

- inspect Servo's current feature flags
- understand platform codec dependencies
- document build requirements
- test licensing implications
- avoid silently making the binary much larger

Do not add media dependencies merely because a website happens to use video.

---

# 55. Licensing and Commercial Use

This project may be commercially distributed.

Before adding a dependency:

1. inspect its license
2. inspect relevant transitive dependencies
3. document important obligations
4. avoid incompatible dependencies
5. preserve required notices

Servo's licensing and dependency licensing should be reviewed before distribution.

Do not assume "Rust crate" means "commercially unrestricted."

---

# 56. Dependency Rules

Every dependency must justify itself.

Before adding one:

- Is it already provided by Servo?
- Is there already an existing project dependency?
- Does it introduce native libraries?
- Does it increase binary size?
- Does it complicate cross-compilation?
- Is it actively maintained?
- Is its license acceptable?

Prefer reuse over dependency proliferation.

---

# 57. Browser Engine Boundary

The most important code review question is:

> **Does this change belong in the browser application, or does Servo already own this behavior?**

If Servo owns it:

```text
Use Servo.
```

If the application owns it:

```text
Implement it outside Servo.
```

Examples:

| Feature                    | Owner       |
| -------------------------- | ----------- |
| HTML parsing               | Servo       |
| CSS parsing                | Servo       |
| DOM                        | Servo       |
| JS runtime                 | Servo       |
| Layout                     | Servo       |
| WebRender                  | Servo       |
| Navigation UI              | Application |
| Address bar                | Application |
| Tabs                       | Application |
| CLI                        | Application |
| HTTP server                | Application |
| Screenshot command         | Application |
| Server limits              | Application |
| Browser profile management | Application |

Do not duplicate ownership.

---

# 58. What Agents Must Never Do

Never:

```text
fork Servo and rewrite large portions without a specific upstream-independent reason
```

Never:

```text
implement custom HTML/CSS/JS rendering because a Servo API is inconvenient
```

Never:

```text
add Chromium "temporarily"
```

Never:

```text
make headless mode a fake hidden GUI without validating the rendering architecture
```

Never:

```text
trust arbitrary server-supplied URLs
```

Never:

```text
disable security checks to make tests pass
```

Never:

```text
commit generated build artifacts
```

Never:

```text
claim a rendering bug is fixed without a regression test
```

---

# 59. When Servo Is Missing a Feature

If Servo does not support a required web feature:

1. Confirm the limitation using current Servo source/docs/tests.
2. Determine whether the feature is required for the product.
3. Check whether Servo already has an experimental implementation.
4. Check whether the feature is on Servo's roadmap.
5. Prefer contributing the missing feature upstream if appropriate.
6. Only implement application-level workarounds when they do not violate web semantics.
7. Never replace Servo with another browser engine.

The project should remain Servo-only.

---

# 60. Upstream Strategy

When a problem is clearly a Servo bug or missing engine feature, consider an upstream contribution.

Good upstream candidates:

- generic HTML/CSS behavior
- Web Platform API
- rendering correctness
- input/event handling
- WebView API
- accessibility
- WebRender behavior

Application-specific behavior stays in this repository.

Maintain a record of upstream patches/workarounds in:

```text
packages/landing/md/docs/project/architecture.md
```

under §Servo Integration Notes and Workarounds.

---

# 61. Agent Loop Engineering

Agents may operate continuously over the task list.

Use:

```text
1. Find highest-priority pending task
2. Read task
3. Inspect repository
4. Inspect Servo API/source if relevant
5. Produce implementation plan
6. Implement
7. Run focused tests
8. Run broader tests
9. Review changes
10. Update task
11. Commit
12. Select next task
```

If blocked:

```text
STOP
 ↓
Document blocker
 ↓
Explain evidence
 ↓
Identify possible paths
```

Do not make random changes indefinitely.

---

# 62. Task Prioritization

Default priority:

```text
P0 — Build/runtime blockers
P1 — Core headed/headless functionality
P2 — Correctness and regression fixes
P3 — Server reliability/security
P4 — Web compatibility
P5 — Performance
P6 — Product UX
P7 — Nice-to-have features
```

A rendering correctness bug should normally outrank a UI polish task.

A server security issue should outrank performance optimization.

---

# 63. Final Architecture

The desired final architecture is:

```text
                         MY BROWSER
                             |
          +------------------+------------------+
          |                  |                  |
       Desktop             CLI               Server
          |                  |                  |
       Shell             Headless            HTTP API
          |                  |                  |
          +------------------+------------------+
                             |
                       Browser Runtime
                             |
                         Servo WebView
                             |
       +---------------------+---------------------+
       |                     |                     |
     Script                Layout              Renderer
       |                     |                     |
   SpiderMonkey        Servo Layout          WebRender
       |                     |                     |
       +---------------------+---------------------+
                             |
                    Rendering Context
                             |
                +------------+------------+
                |                         |
              Window                  Offscreen
                |                         |
             Desktop                  Screenshot
```

The browser application should remain thin.

Servo should remain responsible for the web platform.

---

# 64. Core Success Criteria

The project succeeds when all of these are true:

### Engine

- Servo is the only browser engine.
- No Chromium/WebKit/Gecko fallback exists.
- Servo upgrades are manageable.

### Headed

- A user can open a website.
- Pages render interactively.
- Mouse and keyboard input work.
- Navigation works.

### Headless

- The same browser runtime renders without a visible window.
- Screenshots can be generated.
- The process can run in a server environment appropriate to the target platform.

### Server

- HTTP requests can trigger Servo rendering.
- Requests are isolated.
- Timeouts and resource limits work.
- SSRF protections exist.
- The service can run without a desktop browser UI.

### Engineering

- Rendering behavior is tested.
- Regression tests are maintained.
- Servo integration is isolated.
- AI agents can continue development from task files.
- Documentation reflects the actual implementation.

---

# Final Principles

When uncertain:

### Prefer Servo over reimplementation.

### Prefer one browser core over parallel implementations.

### Prefer correctness over compatibility hacks.

### Prefer tests over assumptions.

### Prefer explicit lifecycle management over sleeps.

### Prefer isolation over shared mutable state.

### Prefer secure defaults over convenience.

### Prefer small application code around a capable engine over rebuilding the engine.

### Prefer upstream Servo improvements when the problem belongs to Servo.

The goal is **not** to build another Chromium.

The goal is:

> **Build the smallest useful browser product around Servo, with one Rust codebase that can run interactively on a desktop and non-interactively as a headless/server rendering runtime.**
