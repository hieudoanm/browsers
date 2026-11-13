---
title: Docker
description: Build and run browserverless serve as a container image.
order: 9
---

# Docker

Two Dockerfiles ship the prebuilt Linux binary on a `debian:trixie-slim`
runtime with the Servo system libraries installed:

| Dockerfile          | Default `PORT` | Notes                                                           |
| ------------------- | -------------- | --------------------------------------------------------------- |
| `Dockerfile`        | `8080`         | `CMD browserverless serve --bind 0.0.0.0:${PORT}`               |
| `docker/Dockerfile` | `10000`        | Adds `XDG_RUNTIME_DIR`/`XDG_CACHE_HOME` for restricted runtimes |

Both run as a non-root `browserverless` user and include a
`HEALTHCHECK` against `GET /api/v1/health`.

## Build and run

```bash
docker build -t browserverless .
docker run --rm -p 8080:8080 browserverless
```

The image downloads `browserverless-x86_64-unknown-linux-gnu` from the
[latest GitHub release](https://github.com/hieudoanm/browsers/releases), so it
is **linux/amd64 only**. On Apple Silicon, build with
`docker buildx build --platform linux/amd64` (emulated) or run the
`macos-aarch64` binary natively instead.

## Configuration

| Variable   | Default | Purpose                       |
| ---------- | ------- | ----------------------------- |
| `PORT`     | `8080`  | Port the HTTP server binds to |
| `RUST_LOG` | `info`  | Logging filter                |

The server is **single-threaded**: it owns one `HeadlessBrowser` and handles
requests serially. Size the container for one page render at a time and scale
with replicas, not threads — see [Architecture](/docs/project/architecture/)
for the Servo process-global constraint.

```bash
docker run --rm -p 8080:8080 -e PORT=9090 browserverless
curl http://localhost:9090/api/v1/health
# {"status":"ok"}
```

Platform-specific guides: [AWS](/docs/deployment/aws/),
[Azure](/docs/deployment/azure/), [GCP](/docs/deployment/gcp/),
[Terraform](/docs/deployment/terraform/), [Heroku](/docs/deployment/heroku/),
[Render](/docs/deployment/render/).
