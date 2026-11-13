---
title: GCP
description: Deploy browserverless serve on Google Cloud Run.
order: 12
---

# GCP

**Cloud Run** is the natural fit: it runs any OCI image, injects `PORT`, and
scales to zero. The root `Dockerfile` works unmodified.

## 1. Build and push

```bash
gcloud builds submit --tag gcr.io/PROJECT_ID/browserverless .
# or, with Artifact Registry:
gcloud builds submit --tag REGION-docker.pkg.dev/PROJECT_ID/browserverless/web:latest .
```

Cloud Build uses the repo `Dockerfile` and produces a **linux/amd64** image —
matching the x86_64 release binary.

## 2. Deploy

```bash
gcloud run deploy browserverless \
  --image gcr.io/PROJECT_ID/browserverless \
  --platform managed \
  --region us-central1 \
  --port 8080 \
  --memory 2Gi \
  --cpu 1 \
  --concurrency 1 \
  --min-instances 0 \
  --allow-unauthenticated
```

Notes:

- `--concurrency 1` matches the server's serial request model — extra
  connections would just queue inside the process anyway.
- `--min-instances 0` scales to zero, but the first request pays a cold start
  (process boot **plus** Servo initialization). Set `--min-instances 1` if
  latency matters.
- `--allow-unauthenticated` exposes the API publicly — the server has no auth.
  Drop it and front the service with IAM/IAP if the endpoint must stay
  private, and read the [server URL policy
  notes](/docs/project/architecture/) before exposing it to untrusted input:
  `serve` is an SSRF-capable renderer.

## 3. Verify

```bash
SERVICE_URL=$(gcloud run services describe browserverless \
  --region us-central1 --format 'value(status.url)')
curl "$SERVICE_URL/api/v1/health"
# {"status":"ok"}
```

Related: [Docker](/docs/deployment/docker/),
[HTTP API](/docs/api/http-api/).
