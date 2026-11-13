---
title: Heroku
description: Deploy browserverless serve on Heroku as a container.
order: 14
---

# Heroku

Heroku has no Rust buildpack for this binary, so deploy the root `Dockerfile`
as a **container** — it already honors the `PORT` Heroku injects.

## 1. One-time setup

```bash
heroku create browserverless-prod
heroku stack:set container -a browserverless-prod
heroku container:login
```

## 2. Build and release

From the repo root (the `Dockerfile` there is the deployable image):

```bash
heroku container:push web -a browserverless-prod
heroku container:release web -a browserverless-prod
```

`heroku container:push` builds `./Dockerfile` for the Heroku-expected
`linux/amd64` platform. The image's `CMD` runs
`browserverless serve --bind 0.0.0.0:${PORT}`, so the dyno listens on the port
Heroku routes to — no `Procfile` needed.

## 3. Dyno size

Servo plus a rendered page needs real memory — the free Eco dyno (512 MB)
will OOM under load. Use **Standard-1X (512 MB) as a floor, Standard-2X
(1 GB) recommended**, and watch `heroku logs --tail` for RSS growth.

```bash
heroku ps:resize web=standard-2x -a browserverless-prod
```

## 4. Verify

```bash
heroku apps:info -a browserverless-prod   # note the web URL
curl https://browserverless-prod.herokuapp.com/api/v1/health
# {"status":"ok"}
```

Each dyno runs one `serve` process and serializes requests internally; Heroku
scales by dyno count, which matches the model well.

Related: [Docker](/docs/deployment/docker/),
[HTTP API](/docs/api/http-api/).
