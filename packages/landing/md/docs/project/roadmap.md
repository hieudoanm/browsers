---
title: Roadmap
description: Phase status, verified milestones, and what comes next.
order: 7
---

# Roadmap

Phase tracking for the browserverless embedder, against `AGENTS.md` §46.
Status as of 2026-11.

| Phase | Name                    | Status                                                                                                                                                                                |
| ----- | ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0     | Repository bootstrap    | ✅ `cargo check/test/fmt/clippy` green (3-crate workspace)                                                                                                                             |
| 1     | Servo smoke test        | ✅ Servo embeds; WebView + SoftwareRenderingContext; local HTML renders                                                                                                                |
| 2     | Headed browser          | ✅ winit window + `WindowRenderingContext` + paint/present loop                                                                                                                        |
| 3     | Navigation shell        | ⚠️ minimal: scroll + resize forwarded; no back/forward/reload/address-bar                                                                                                              |
| 4     | Headless rendering      | ✅ offscreen PNG; verified against mvp + real URLs (see [architecture](/docs/project/architecture/))                                                                                   |
| 5     | Deterministic rendering | ⚠️ fixed viewport + scale; fixtures exist; no image-diff comparison yet, no local HTTP test server                                                                                     |
| 6     | CLI                     | ⚠️ subcommands `open`/`headless`/`screenshot`/`scrape`/`memory`/`serve`/`mcp`; `--script`/`--user-agent`/`--version` not wired                                                         |
| 7     | Server                  | ⚠️ `serve` subcommand live (health/version/scrape/screenshot over one serialized worker) — see [architecture](/docs/project/architecture/); separate server crate + hardening deferred |
| 8     | Server hardening        | ❌                                                                                                                                                                                     |
| 9     | Browser state           | ❌                                                                                                                                                                                     |
| 10    | Multiple pages/tabs     | ❌ (blocked: multi-page in one context is unreliable — see [architecture](/docs/project/architecture/) Servo Integration Notes §6)                                                     |
| 11+   | Product features        | ❌                                                                                                                                                                                     |

## Verified milestone (Phase 4 acceptance)

```bash
cargo run -p browsers-cli -- screenshot \
  file://$(pwd)/crates/browsers-cli/tests/fixtures/rendering/mvp.html \
  --output /tmp/mvp.png --width 800 --height 600 --timeout 60000
```

## Immediate next work (priority order)

1. **Server hardening** (Phase 8) — separate `server` crate/bin, SSRF protections,
   concurrency/response limits, JSON + PNG render endpoints, graceful shutdown.
2. **Deterministic rendering** (Phase 5) — local HTTP test server + image-diff
   golden tests for the existing fixtures.
3. **Headless CLI flags** (Phase 6) — `--script`, `--user-agent`, `--version`,
   and honoring the `open` URL argument.
4. **Diagnose multi-page contamination** (Phase 10 prerequisite) — second
   `new_page()` in a process renders the first page's content; needs a
   Servo-internals trace before tabs/pools.

## The experimental engine crates

`crates/browsers-{css,dom,gui,html,layout,network,paint,renderer,style}` are a
from-scratch HTML/CSS/layout/paint prototype explored before the project
committed to embedding Servo. They are preserved under conventional names for
reference and possible future use (e.g. a non-Servo fallback renderer), but
they are intentionally **not** workspace members: several dependencies are not
yet wired and `browsers-gui` depends on a planned `browsers-browser` crate
that does not exist yet. Their original task breakdown (tasks 001–018) has
been removed; the crates themselves remain the reference.
