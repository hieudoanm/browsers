# Contributing

Everything below is verified against the current implementation. The Servo
version is pinned in `Cargo.lock`; Servo's embedding API is pre-1.0 and changes
between releases — always re-verify against the pinned source (see the
[architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/)
§Servo Integration Notes).

## Setup

Prerequisites:

- Rust toolchain (edition 2021, resolver 2; `rust-toolchain.toml` pins stable +
  rustfmt/clippy). The project has been built on macOS arm64.
- Platform graphics stack required by Servo (surfman backend, etc.). macOS
  works out of the box; Linux needs the package list in CI
  (`.github/workflows/ci.yaml`) or the landing
  [getting-started](https://hieudoanm.github.io/browsers/docs/guide/getting-started/)
  docs. Headed mode additionally needs a windowing session.

## Build

```bash
cargo build                        # workspace debug build (3 member crates)
cargo build -p browsers-cli        # the binary package
cargo build --release
```

`browserverless` is the bin **name**; the compile **package** is `browsers-cli`.

## Commands

| Command                                                           | Purpose                               |
| ----------------------------------------------------------------- | ------------------------------------- |
| `cargo build`                                                     | Workspace debug build                 |
| `cargo build --release`                                           | Release build                         |
| `cargo run -p browsers-cli -- screenshot <url> [opts]`      | Headless screenshot of a URL          |
| `cargo run -p browsers-cli -- scrape <url> [-o page.html]`  | Dump the full HTML of a URL           |
| `cargo run -p browsers-cli -- headless <url> [--timeout N]` | Headless load-and-exit                |
| `cargo run -p browsers-cli -- memory <url> [--observe N]`   | Measure peak RSS + duration           |
| `cargo run -p browsers-cli -- serve --bind 127.0.0.1:8080`  | Serve the HTTP rendering API          |
| `cargo run -p browsers-cli -- mcp serve`                   | Serve MCP tools over stdio            |
| `cargo run -p browsers-cli -- open`                         | Headed window (hardcoded example.com) |
| `RUST_LOG=debug cargo run ...`                                    | Verbose Servo/embedder logs           |

CLI screenshot flags: `--output` (default `screenshot.png`), `--width` (1280),
`--height` (720), `--timeout` (30 000 ms, pass to `load_timeout_ms`). The URL is
positional and must be absolute (`file://`, `http://`, `https://`). Server
flags: `--bind` (default 127.0.0.1:8080), `--port` (overrides the port in
`--bind`), `--width`/`--height` (serve viewport, default 1280x720), `--timeout`
(30 000). `make screenshot` / `make gui` run the CLI against the bundled
fixtures.

## Test / Check (must stay green before finishing a change)

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs the same gates via `.github/workflows/ci.yaml` (see the
[packaging docs](https://hieudoanm.github.io/browsers/docs/project/packaging/)).

## Landing page

`packages/landing` is a Next.js static export (Tailwind + DaisyUI) deployed to
GitHub Pages. Docs content lives in `packages/landing/md/docs/**/*.md` — update it
when behaviour changes. See `packages/landing/AGENTS.md`.

## Coding Conventions

Follow the repo-wide conventions in `AGENTS.md`. In short for Rust:

- `Result<T, E>` for fallible functions, never `panic!`; `Option<T>` over
  sentinel values
- `match` over `if let` chains (exhaustive, compiler-verified)
- `thiserror` for error types in libraries, `#[derive(Debug, Clone,
  PartialEq)]` liberally
- Prefer `&str` over `&String` in function params; iterators over indexed loops
- Keep `unsafe` in small, audited blocks with `// SAFETY:` comments
- `cargo clippy -- -D warnings` as the lint gate

Servo-specific rules (`AGENTS.md` §44): inspect the exact Servo version in
`Cargo.lock`, prefer the public embedding API, isolate unavoidable Servo
internals at the single boundary (`crates/browsers-core/src/lib.rs`), document
every workaround in the
[architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/)
§Servo Integration Notes, and add a regression test.

## Testing Conventions

- Colocate `*_test.rs` files in the same crate (Rust convention).
- **The headless rendering test is a single test.** Servo's `opts`/`prefs` are
  process-global; a second `HeadlessBrowser::new` in the same process panics
  (`Already initialized`). Run the full pipeline with
  `cargo test -p browsers-headless`.
- **The serve test is likewise a single test.**
  `crates/browsers-cli/tests/serve_api.rs` exercises
  health/version/scrape/screenshot/scheme-blocking/400/404/405 in one process.
- Screenshot correctness is asserted against
  `crates/browsers-cli/tests/fixtures/rendering/mvp.html` (golden geometry) and
  real rendered pixels — not empty/blank images.

## Stale crate directories (dead code)

`crates/browsers-css`, `crates/browsers-dom`, `crates/browsers-gui`,
`crates/browsers-html`, `crates/browsers-layout`, `crates/browsers-network`,
`crates/browsers-paint`, `crates/browsers-renderer`, `crates/browsers-style`
are leftovers from a prototyping phase where the project had its own small
engine. They are **not** workspace members (see root `Cargo.toml`) and do not
compile in the current build. They are safe to delete, but do not reintroduce
them — see `AGENTS.md` §0–§58 (Servo-only; never implement a second engine).
`crates/browsers-cli/tests/fixtures/css/*` and
`crates/browsers-cli/tests/fixtures/html/*` are fixtures for that prototype;
`crates/browsers-cli/tests/fixtures/rendering/mvp.html` is current and used for
the MVP acceptance test.

## Troubleshooting

### "all-white screenshot"

Symptoms: saved PNG is uniformly white; `readback distinct_colors=1`.
Cause to rule out first: the navigation never reached the page. See the
[architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/)
§Servo Integration Notes §1 — pass the URL via `WebViewBuilder::url()` on
`new_page`, never relying on an uncoupled `webview.load()` right after build.

### `LoadUrl for unknown browsing context` (WARN from constellation)

A `webview.load()` was issued before the constellation registered the browsing
context; the navigation is **dropped silently**, leaving the initial
`about:blank` (white). Do not add more `load()` calls; pre-spin the event loop
before issuing a load and prefer builder-supplied URLs for the initial
navigation.

### `readback after present` is black

Expected. The software surface is double-buffered (surfman); `present()`
swaps in a cleared buffer. Read the surface **before** present, or use
`take_screenshot` (the PNG path) which reads the WebRender scene directly.

### `Cropping texture upload Box2D((0, 0), (0, 1))` warnings

Harmless noise from webrender debug asserts on near-empty crop rects. Not the
cause of rendering bugs.

### `UNSUPPORTED ... GL_* is unloadable and bound to sampler type (Float)`

Harmless GL-context noise on macOS; screenshots still render correctly.

### Timeout on heavy pages

`HeadlessConfig::load_timeout_ms` (or CLI `--timeout`) defaults to 30 s.
Debug-build Servo rendering of heavyweight sites (e.g. Wikipedia) benefits from
60–120 s. The regression test uses 60 s.

### `Already initialized: Opts {...}` panic

You attempted to construct a **second** Servo instance in the same process.
Servo's `opts`/`prefs` are process-global and can only be set once. Run each
Servo instance in its own process. This also explains why the headless
rendering test is a single test — the second `HeadlessBrowser::new` would
panic.

## Servo upgrades

Per `AGENTS.md` §2/§41:

1. Change the pin in `Cargo.toml` / update `Cargo.lock`.
2. `cargo build --workspace`.
3. Fix embedding API breakages at the single boundary
   (`crates/browsers-core/src/lib.rs`).
4. `cargo test --workspace`, then real-URL smoke tests (see the
   [architecture docs](https://hieudoanm.github.io/browsers/docs/project/architecture/)
   §Compatibility).
5. Record any new workaround in the architecture docs §Servo Integration
   Notes; update the compatibility notes.

## Docs

- `packages/landing/md/docs/` — the published documentation site content
  (deployed to GitHub Pages). Update it when the CLI, HTTP API, MCP tools,
  architecture, or roadmap changes.
- `README.md` — update when adding or changing CLI commands.
- `AGENTS.md` — the project spec; keep phase/checklist status in sync with
  what actually ships.
