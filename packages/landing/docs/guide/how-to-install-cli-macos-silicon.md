---
title: Install the CLI on macOS (Apple Silicon)
description:
  Download and install the browserverless binary for macOS (Apple Silicon).
order: 60
---

# How to install the browserverless CLI on macOS (Apple Silicon)

For Macs with an M-series chip (arm64). The release asset is named
`cli-browsers-*` but the binary inside is `browserverless`.

## Download

1. Open the [downloads page](https://hieudoanm.github.io/browsers/downloads/)
   and download **macOS (Apple Silicon)** under **CLI** —
   `cli-browsers-aarch64-apple-darwin`.
2. Make it executable, rename it to `browserverless`, and move it onto your
   `PATH`:

```sh
chmod +x cli-browsers-aarch64-apple-darwin
mv cli-browsers-aarch64-apple-darwin ~/.local/bin/browserverless
```

3. Verify:

```sh
browserverless --version
```

## Troubleshooting

- `browserverless: command not found` — add the install directory to your
  `PATH`.
- Linux binaries need the usual Servo system libraries (fontconfig, freetype,
  harfbuzz, GTK3, EGL/GL) — the same set the project's Dockerfile installs.
- Gatekeeper warning — the binary is unsigned. Right-click → Open, or run
  `xattr -d com.apple.quarantine` on it.
