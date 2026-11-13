---
title: AWS
description: Deploy browserverless serve on AWS with ECS Fargate and ECR.
order: 10
---

# AWS

The simplest production shape is **ECS Fargate + ECR**: build the root
`Dockerfile`, push to ECR, and run it as a Fargate service behind a load
balancer or service discovery.

## 1. Build and push the image

```bash
aws ecr create-repository --repository-name browserverless
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin \
  123456789012.dkr.ecr.us-east-1.amazonaws.com

docker build --platform linux/amd64 -t browserverless .
docker tag browserverless \
  123456789012.dkr.ecr.us-east-1.amazonaws.com/browserverless:latest
docker push \
  123456789012.dkr.ecr.us-east-1.amazonaws.com/browserverless:latest
```

The release binary is x86_64-only, so Fargate tasks must use the `X86_64`
runtime platform (the default).

## 2. Task definition

Key fields for `browserverless serve`:

```json
{
  "family": "browserverless",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "runtimePlatform": { "cpuArchitecture": "X86_64" },
  "containerDefinitions": [{
    "name": "browserverless",
    "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/browserverless:latest",
    "portMappings": [{ "containerPort": 8080, "protocol": "tcp" }],
    "environment": [
      { "name": "PORT", "value": "8080" },
      { "name": "RUST_LOG", "value": "info" }
    ],
    "logConfiguration": {
      "logDriver": "awslogs",
      "options": {
        "awslogs-group": "/ecs/browserverless",
        "awslogs-region": "us-east-1",
        "awslogs-stream-prefix": "serve"
      }
    },
    "healthCheck": {
      "command": ["CMD-SHELL", "wget -q -O /dev/null http://localhost:8080/api/v1/health || exit 1"],
      "interval": 30, "timeout": 5, "retries": 3, "startPeriod": 15
    }
  }]
}
```

Give the task at least **1 vCPU / 2 GB** — Servo plus a rendered page is not
cheap. The container already runs as non-root and ships its own `HEALTHCHECK`.

## 3. Service

Create an ECS service with `desiredCount` matching your throughput needs. The
server serializes requests internally, so **horizontal scaling is the only
lever** — each task handles one render at a time. Put an Application Load
Balancer in front for HTTPS termination, or use Service Connect for
VPC-internal callers.

```bash
aws ecs create-service \
  --cluster my-cluster \
  --service-name browserverless \
  --task-definition browserverless \
  --desired-count 2 \
  --launch-type FARGATE \
  --network-configuration "awsvpcConfiguration={subnets=[subnet-...],securityGroups=[sg-...],assignPublicIp=DISABLED}"
```

## Alternative: App Runner

If you want zero cluster management, App Runner can deploy the same ECR image
directly (`apprunner.amazonaws.com` service with port 8080). It autoscales
per-instance but bills per vCPU/GB-second — fine for bursty scraping, costly
for steady traffic.

Related: [Docker](/docs/deployment/docker/),
[Terraform (ECS example)](/docs/deployment/terraform/),
[HTTP API](/docs/api/http-api/).
