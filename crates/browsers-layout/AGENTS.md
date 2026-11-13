# AGENTS.md — browsers-layout

## Overview

Experimental layout crate of the pure-Rust browser engine prototype
(originally specified in task docs 002–018, since removed). It is **not** a Cargo workspace member and is not
compiled by `cargo build --workspace`.

## Conventions

- When this crate is eventually wired in, follow the workspace conventions:
  edition 2021, exact-pinned deps via the root `[workspace.dependencies]`,
  `thiserror` for errors, tests in `tests/` (not inline `#[cfg(test)]`).
- Crate-to-crate references use the `browsers_*` names; keep path deps
  pointed at `../browsers-<name>`.
- Do not add Servo or winit dependencies here — the prototype engine is
  deliberately independent of the Servo embedder in `browsers-core`.
