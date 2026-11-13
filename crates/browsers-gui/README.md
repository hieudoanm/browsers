# browsers-gui

Experimental gui component of the browserverless pure-Rust engine
prototype.

Desktop window shell: `GuiBrowser` wrapping winit + softbuffer to display rendered content. Depends on a planned `browsers-browser` crate that does not exist yet — WIP.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
