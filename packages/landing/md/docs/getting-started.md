---
title: Getting started
description: Install a prebuilt binary or build from source, then take your first screenshot.
order: 2
---

# Getting started

## Install a prebuilt binary

Release builds are published to the
[GitHub Releases](https://github.com/hieudoanm/browsers/releases) page:

```bash
gh release download latest \
  --repo hieudoanm/browsers \
  --pattern "browserverless-*"
chmod +x browserverless-*
```

Prebuilt binaries are currently published for `linux-x86_64` and
`macos-aarch64`. Linux binaries need the usual Servo system libraries
(fontconfig, freetype, harfbuzz, GTK3, EGL/GL — the same set the Dockerfile
installs).

Or via a container image — two Dockerfiles ship the prebuilt binary on a
`debian:trixie-slim` runtime:

```bash
# Root Dockerfile (default port 8080)
docker build -t browserverless-server .
docker run -p 8080:8080 browserverless-server

# docker/Dockerfile (default port 10000, correct HOME/XDG dirs for headless)
cd docker
docker build -t browserverless-server .
docker run -p 10000:10000 browserverless-server
```

Both images serve `GET /api/v1/health` (also used as the `HEALTHCHECK`). See
[Packaging & releases](packaging) for the full runtime library set.

## Build from source

Requirements: a stable Rust toolchain (`rust-toolchain.toml` pins `stable` with
rustfmt + clippy) and a C toolchain. On Linux, install the Servo system
dependencies first:

```bash
sudo apt-get install -y --no-install-recommends \
  pkg-config cmake libfontconfig1-dev libfreetype6-dev libharfbuzz-dev \
  libglib2.0-dev libdbus-1-dev libpango1.0-dev libcairo2-dev libgtk-3-dev \
  libx11-dev libxcb-shape0-dev libxcb-xfixes0-dev libwayland-dev \
  libwayland-bin libegl-dev libgl-dev zlib1g-dev
```

Then:

```bash
cargo build --release -p browsers-cli
# binary at target/release/browserverless
```

A debug build works but is dramatically slower — use `--release` for anything
beyond a smoke test.

## First screenshot

```bash
browserverless screenshot https://example.com \
  --output example.png --width 1280 --height 720 --timeout 60000
```

Or render a local file:

```bash
browserverless screenshot "file://$(pwd)/page.html" -o page.png
```

The URL must be absolute (`http://`, `https://`, or `file://`).

## Run the test suite

```bash
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

> Servo's configuration is process-global, so tests that spin up a browser are
> consolidated into single integration tests per crate — expect a small number
> of long-running tests rather than many fast ones.
