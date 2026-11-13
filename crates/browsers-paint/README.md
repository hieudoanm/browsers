# browsers-paint

Experimental paint component of the browserverless pure-Rust engine
prototype.

Display list: `DisplayCommand`, `DisplayList`, `build_display_list` — converts layout boxes into ordered paint operations.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
