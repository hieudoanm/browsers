---
title: Packaging & releases
description: How browserverless is built, containerized, and published — binaries, Docker images, and the CI pipeline.
order: 8
---

# Packaging & releases

Browserverless ships as a Cargo-built release binary and Debian-based OCI
container images, published through a rolling GitHub Release.

## Binary

`cargo build --release` produces the `browserverless` binary:

```bash
cargo build --release
# target/release/browserverless
```

Browserverless is **dynamically linked** against Servo's graphics stack
(surfman/GL, fontconfig/harfbuzz, GTK/X11/Wayland bits). There is no static
build; system dependencies must be present at runtime — see
[Getting started](getting-started) for the package list, and the `Dockerfile`
for the exact runtime set used in a container.

## Downloads

CI publishes prebuilt binaries to the rolling
[`latest` release](https://github.com/hieudoanm/browsers/releases):

```txt
browserverless-x86_64-unknown-linux-gnu
browserverless-aarch64-apple-darwin
```

```bash
chmod +x browserverless-x86_64-unknown-linux-gnu
./browserverless-x86_64-unknown-linux-gnu serve --bind 127.0.0.1:8080
```

Or via the CLI:

```bash
gh release download latest --repo hieudoanm/browsers --pattern "browserverless-*"
```

## Container

Two Dockerfiles exist, both based on `debian:trixie-slim` and both downloading
the prebuilt binary from the rolling release:

| File                | Default port | Notes                                                                                                         |
| ------------------- | ------------ | ------------------------------------------------------------------------------------------------------------- |
| `Dockerfile`        | 8080         | Minimal runtime: system graphics libs + `browserverless` user.                                                |
| `docker/Dockerfile` | 10000        | Adds `HOME`, `XDG_RUNTIME_DIR`, `XDG_CACHE_HOME` for headless runtime; non-root user with a home + cache dir. |

Runtime libraries installed by both: `libfontconfig1`, `libfreetype6`, `libgl1`,
`libegl1`, `libx11-6`, `libxcb-shape0`, `libxcb-xfixes0`, `libglib2.0-0`,
`libgtk-3-0`, `libharfbuzz0b`, `libdbus-1-3`, `libpango-1.0-0`, `libcairo2`,
`ca-certificates`, `wget`.

Both expose the HTTP API and health-check it:

```text
EXPOSE ${PORT}
HEALTHCHECK CMD wget --no-verbose --tries=1 --spider http://localhost:${PORT}/api/v1/health
```

### Build the image

```bash
# Root Dockerfile (port 8080)
docker build -t browserverless-server .
docker run -p 8080:8080 browserverless-server

# docker/Dockerfile (port 10000)
cd docker
docker build -t browserverless-server .
docker run -p 10000:10000 browserverless-server
```

Both serve `GET /api/v1/health` (also used as the image `HEALTHCHECK`).

## CI Pipeline

`.github/workflows/ci.yaml` runs the Rust gates plus a release pipeline:

| Job             | What it does                                                                                                                                                                          |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cli-check`     | `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test` on ubuntu.                                                                              |
| `cli-build`     | `cargo build --release -p browsers-cli` on ubuntu + macOS (non-PR only); uploads `browserverless-<triple>` artifacts.                                                        |
| `release`       | On `master`: downloads both artifacts and (re)creates the rolling GitHub Release `latest`.                                                                                            |
| `landing-*`     | Typecheck/lint/build the Next.js static export and deploy `packages/landing/out` to GitHub Pages.                                                                                    |

The release job force-deletes and recreates the release/tag on every run, so
the download URLs above always point at the newest build.

## Release checklist

1. `cargo fmt --all --check` clean
2. `cargo clippy --workspace --all-targets -- -D warnings` clean
3. `cargo test --workspace` green (includes the headless rendering + serve API tests)
4. `cargo build --release` succeeds and `./target/release/browserverless serve --help` runs
5. `docker build -t browserverless-server .` succeeds and the image answers
   `GET /api/v1/health` with `{"status":"ok"}`

## License

GPL-3.0 (see `LICENSE` in the repository).
