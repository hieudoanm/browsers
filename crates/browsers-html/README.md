# browsers-html

Experimental html component of the browserverless pure-Rust engine
prototype.

HTML parser: `parse_html(&str) -> Dom` built on html5ever with a custom `TreeSink` that constructs `browsers-dom` nodes directly.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
