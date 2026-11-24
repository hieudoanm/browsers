---
title: Install the CLI on Linux
description: Download and install the browserverless binary for Linux.
order: 60
---

# How to install the browserverless CLI on Linux

For x86_64 Linux distributions. The release asset is named `cli-browsers-*` but
the binary inside is `browserverless`.

## Download

1. Open the [downloads page](https://hieudoanm.github.io/browsers/downloads/)
   and download **Linux** under **CLI** —
   `cli-browsers-x86_64-unknown-linux-gnu`.
2. Make it executable, rename it to `browserverless`, and move it onto your
   `PATH`:

```sh
chmod +x cli-browsers-x86_64-unknown-linux-gnu
mv cli-browsers-x86_64-unknown-linux-gnu ~/.local/bin/browserverless
```

3. Verify:

```sh
browserverless --version
```

## Troubleshooting

- `browserverless: command not found` — add the install directory to your
  `PATH`.
- Missing shared libraries — install the Servo runtime deps (fontconfig,
  freetype, harfbuzz, GTK3, EGL/GL); see the project's Dockerfile for the list.
