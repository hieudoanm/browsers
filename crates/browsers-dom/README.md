# browsers-dom

Experimental dom component of the browserverless pure-Rust engine
prototype.

DOM tree data structures: `Dom`, `Node`, `NodeId`, `TreeNode`, element/text/comment data, `QuirksMode`. Flat arena storage with parent/child/sibling links.

## Status

**Not a workspace member** — this crate is preserved for the engine
experimentation from an earlier prototype phase and is intentionally excluded from
`cargo build --workspace`. Several of its dependencies are not yet wired into
the workspace manifest.
