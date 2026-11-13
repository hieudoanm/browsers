# browsers-style

Experimental style component of the browserverless pure-Rust engine
prototype.

Style resolution: `ComputedStyle` (with `inherited()`), `StyleContext::compute(dom, stylesheet)` matching CSS rules against DOM nodes and producing per-node computed styles.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
