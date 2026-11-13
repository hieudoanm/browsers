# browsers-headless

Offscreen browser execution for browserverless — load a URL, wait for render,
and produce a PNG screenshot, an HTML dump, or a structured `ScrapeResult` /
`ScreenshotResult` for the HTTP and MCP layers.

## Public API

- `HeadlessConfig` — viewport, `load_timeout_ms` (30s), `wait_after_load_ms` (3s).
- `HeadlessBrowser::new(config)` — builds the underlying `BrowserContext`.
- `render_url(url)` — navigate + wait; returns the `Page`.
- `screenshot_url(url, path)` — render and write a PNG.
- `screenshot_bytes[_with_timeout](url)` — PNG bytes for the HTTP API.
- `scrape[_with_timeout](url)` — rendered HTML + final URL + title +
  load status for `/api/v1/scrape` and MCP.
- `dump_html(url)`, `spin_event_loop()`.

## Notes

- Depends only on `browsers-core` — the dependency direction is
  `cli → headless → core → servo`.
- Timing-sensitive pages may need a larger `wait_after_load_ms`; heavy pages
  (e.g. Wikipedia) may need `load_timeout_ms` ≈ 120s on debug builds.
- `tests/headless_rendering.rs` runs the whole pipeline in one test because
  Servo is process-global.
