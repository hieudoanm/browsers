# browsers-network

Experimental network component of the browserverless pure-Rust engine
prototype.

Resource loading: `ResourceLoader`, `Resource`, `ResourceType`, `NetworkError` — fetches URLs via reqwest and classifies HTML/CSS/other resources; also `load_html`/`load_css` injection helpers.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
