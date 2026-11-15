## Browserverless — latest build

A lightweight browser built on the Servo engine: one Rust codebase, two
execution modes — headed desktop browser and headless rendering
(screenshots, automation, server API).

### Download

| Platform | Asset |
| --- | --- | 
| Linux x86_64 | `browserverless-x86_64-unknown-linux-gnu` |
| macOS Apple Silicon | `browserverless-aarch64-apple-darwin` |

### Quick start

```sh
browserverless https://example.com          # headed mode
browserverless --headless https://example.com -o shot.png
browserverless-server --bind 0.0.0.0:8080   # render API
```

### Documentation

- [README](https://github.com/hieudoanm/browsers#readme)
- [Docs site](https://hieudoanm.github.io/browsers/docs/)
