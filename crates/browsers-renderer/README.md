# browsers-renderer

Experimental renderer component of the browserverless pure-Rust engine
prototype.

Software rasterizer: `SoftwareRenderer`, `RenderOptions`, `BitmapFont` (built-in bitmap font), `render`/`save_png`/`into_buffer` producing `image::RgbaImage`.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
