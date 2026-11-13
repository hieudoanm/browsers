---
title: Azure
description: Deploy browserverless serve on Azure with Container Instances or Azure Container Apps.
order: 11
---

# Azure

Two good fits, depending on how much orchestration you want:

- **Azure Container Instances (ACI)** — fastest path, one `az` command, no
  cluster to manage.
- **Azure Container Apps (ACA)** — KEDA-scaled containers with revisions,
domains, and Dapr support when you need autoscaling.

Both run the same image; build it once and push to **Azure Container Registry**
(ACR).

## 1. Build and push to ACR

```bash
az group create -n rg-browserverless -l eastus
az acr create -n myregistry -g rg-browserverless --sku Basic
az acr build -r myregistry -t browserverless:latest \
  --platform linux/amd64 .
```

`az acr build` runs the root `Dockerfile` in Azure — no local Docker or
`buildx` needed.

## 2. Azure Container Instances

```bash
az container create \
  -g rg-browserverless \
  -n browserverless \
  --image myregistry.azurecr.io/browserverless:latest \
  --registry-login-server myregistry.azurecr.io \
  --registry-username myregistry \
  --registry-password "$(az acr credential show -n myregistry --query 'passwords[0].value' -o tsv)" \
  --ports 8080 \
  --cpu 1 --memory 2 \
  --dns-name-label browserverless-unique \
  --environment-variables PORT=8080 RUST_LOG=info
```

ACI gives the container a public FQDN on port 8080:
`http://browserverless-unique.eastus.azurecontainer.io`. Add
`--secure-environment-variables` for anything sensitive. Probe
`/api/v1/health` for readiness — the image already declares it as a
`HEALTHCHECK`, which ACI surfaces as container state.

## 3. Azure Container Apps (production)

```bash
az containerapp env create -n cae-browsers -g rg-browserverless -l eastus
az containerapp create \
  -n browserverless -g rg-browserverless \
  --environment cae-browsers \
  --image myregistry.azurecr.io/browserverless:latest \
  --target-port 8080 \
  --ingress external \
  --min-replicas 1 --max-replicas 10 \
  --cpu 1 --memory 2Gi
```

ACA injects `PORT` itself; the image's `CMD` already honors it. Because the
server serializes requests, keep `--min-replicas` at the concurrency level you
actually need rather than relying on per-request scale-out — each replica is
one render at a time.

Related: [Docker](/docs/deployment/docker/),
[HTTP API](/docs/api/http-api/).
