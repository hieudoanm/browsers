# browsers-core

Servo integration boundary for browserverless — the only crate that knows
about the Servo API. Owns `BrowserContext`, `Page`, `PageDelegate`, and the
`PageHandle` state shared with the delegate.

## Public API

- `BrowserConfig` — viewport size + `device_scale_factor`.
- `BrowserContext::new(config)` — builds a `SoftwareRenderingContext` and the
  process-global `Servo` instance.
- `BrowserContext::new_page(url)` — builds a `WebView` with the URL set via
  `WebViewBuilder::url()` (required — see below).
- `Page` — `goto`, `wait_for_load`, `wait_for_render`, `screenshot`,
  `save_screenshot`, `surface_readback`, `present`, `pixel_summary`,
  `evaluate_javascript`.

## Hard rules

- **One `BrowserContext` per process.** Servo `opts` are process-global and
  panic on second initialization — never construct two.
- **Initial URL goes through `WebViewBuilder::url()`.** Navigating with
  `webview.load()` after build races the constellation (all-white screenshots).
- One page per context is the supported model; multi-page is unreliable.

See `https://hieudoanm.github.io/browsers/docs/project/architecture/` §Servo Integration Notes for the full write-up.
