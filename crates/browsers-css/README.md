# browsers-css

Experimental css component of the browserverless pure-Rust engine
prototype.

CSS parsing and representation: `Stylesheet`, `CssRule`, `Selector`/`SelectorPart`/`SelectorKind`/`Combinator`, `Declaration`/`Property`, `Specificity`, `Color` (hex/named/rgb/rgba).

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
