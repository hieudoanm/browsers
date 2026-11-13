---
title: Terraform
description: Provision browserverless serve on AWS ECS Fargate with Terraform.
order: 13
---

# Terraform

A compact AWS ECS Fargate deployment: ECR repository, CloudWatch log group,
task definition, and service. Assumes the `hashicorp/aws` provider and that
you have already pushed the image (see [AWS](/docs/deployment/aws/) for the
`docker build`/`push` steps).

```hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
}

provider "aws" {
  region = "us-east-1"
}

variable "image" {
  description = "Full ECR image URI, e.g. 123456789012.dkr.ecr.us-east-1.amazonaws.com/browserverless:latest"
  type        = string
}

resource "aws_ecr_repository" "browserverless" {
  name = "browserverless"
}

resource "aws_cloudwatch_log_group" "browserverless" {
  name              = "/ecs/browserverless"
  retention_in_days = 30
}

resource "aws_ecs_cluster" "browserverless" {
  name = "browserverless"
}

resource "aws_ecs_task_definition" "browserverless" {
  family                   = "browserverless"
  network_mode             = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                      = "1024"
  memory                   = "2048"
  execution_role_arn       = aws_iam_role.exec.arn
  task_role_arn            = aws_iam_role.task.arn

  runtime_platform {
    cpu_architecture        = "X86_64" # release binary is x86_64-only
    operating_system_family = "LINUX"
  }

  container_definitions = jsonencode([{
    name      = "browserverless"
    image     = var.image
    essential = true
    portMappings = [{
      containerPort = 8080
      protocol      = "tcp"
    }]
    environment = [
      { name = "PORT", value = "8080" },
      { name = "RUST_LOG", value = "info" }
    ]
    logConfiguration = {
      logDriver = "awslogs"
      options = {
        "awslogs-group"         = aws_cloudwatch_log_group.browserverless.name
        "awslogs-region"        = "us-east-1"
        "awslogs-stream-prefix" = "serve"
      }
    }
    healthCheck = {
      command     = ["CMD-SHELL", "wget -q -O /dev/null http://localhost:8080/api/v1/health || exit 1"]
      interval    = 30
      timeout     = 5
      retries     = 3
      startPeriod = 15
    }
  }])
}

resource "aws_ecs_service" "browserverless" {
  name            = "browserverless"
  cluster         = aws_ecs_cluster.browserverless.id
  task_definition = aws_ecs_task_definition.browserverless.arn
  desired_count   = 2 # one render at a time per task — scale horizontally
  launch_type     = "FARGATE"

  network_configuration {
    subnets         = var.private_subnet_ids
    security_groups = [var.service_security_group_id]
    assign_public_ip = false
  }
}
```

You will also need the usual IAM roles (`aws_iam_role.exec` with the
`AmazonECSTaskExecutionRolePolicy`, `aws_iam_role.task` for app permissions)
and subnet/SG variables — the snippet above omits them for brevity.

Because `browserverless serve` processes requests serially, `desired_count`
is your concurrency knob: each task handles one render at a time.

Related: [AWS](/docs/deployment/aws/),
[Docker](/docs/deployment/docker/).
