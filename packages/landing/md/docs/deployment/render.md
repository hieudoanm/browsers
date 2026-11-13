---
title: Render
description: Deploy browserverless serve on Render with a render.yaml blueprint.
order: 15
---

# Render

Render deploys any repo with a `Dockerfile` as a **Docker web service**. The
root `Dockerfile` is already compatible: it honors `PORT`, runs non-root, and
ships a health check.

## render.yaml

Commit this blueprint at the repo root (or paste it into the Render dashboard
as a Blueprint):

```yaml
services:
  - type: web
    name: browserverless
    runtime: docker
    dockerfilePath: ./Dockerfile
    plan: standard      # 512 MB — bump to 'pro' (2 GB+) under real load
    healthCheckPath: /api/v1/health
    autoDeploy: true
    envVars:
      - key: RUST_LOG
        value: info
```

Render builds `./Dockerfile` on their builders (linux/amd64 — matching the
release binary), injects `PORT`, and routes traffic to it. The image's `CMD`
expands `${PORT}` itself, so no command override is needed.

## Notes

- **Health checks**: `/api/v1/health` returns `200 {"status":"ok"}` without
  rendering anything, so it stays cheap even while a scrape is in flight.
- **Scaling**: `serve` is single-threaded — one render at a time per
  instance. Use Render's instance count (or a `pro` plan with more RAM) for
  throughput; scaling CPU alone does not help.
- **Cold starts**: Render spins down idle services on lower plans; the first
  request after idle pays process boot plus Servo init.
- **Public exposure**: the API has no authentication — anyone with the URL can
  make the service fetch arbitrary URLs (SSRF). If that is unacceptable, put
  it behind Render's IP allowlist or a private service and read the
  [URL-policy notes](/docs/project/architecture/) first.

Related: [Docker](/docs/deployment/docker/),
[HTTP API](/docs/api/http-api/).
