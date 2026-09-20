---
name: deployment
description: "Deployment engineering — CI/CD pipelines, containerization, orchestration, and release management"
version: "1.0.0"
author: "NeoTrix"
triggers: "deploy|ci|cd|docker|kubernetes|k8s|container|pipeline|release|rollback|infra|terraform"
---

# Deployment — CI/CD, Docker, Kubernetes

End-to-end deployment pipeline from code commit to production. Automate everything, verify everything.

## CI/CD Pipeline Stages

```
Code → Build → Test → Scan → Package → Deploy → Verify
  │       │       │       │         │         │        │
  │       │       │       │         │         │        └─ Smoke tests, canary
  │       │       │       │         │         └────────── Staging, prod
  │       │       │       │         └──────────────────── Docker, OCI
  │       │       │       └────────────────────────────── SAST, DAST, deps
  │       │       └────────────────────────────────────── Unit, integration
  │       └────────────────────────────────────────────── Compile, lint
  └────────────────────────────────────────────────────── Trigger
```

## Docker Best Practices

### Multi-Stage Build
```dockerfile
# Build stage
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/myapp /usr/local/bin/
CMD ["myapp"]
```

### Image Optimization
- Use specific base image tags (not `latest`)
- Minimize layers (combine RUN commands)
- Use `.dockerignore` to exclude build artifacts
- Run as non-root user
- Scan images for vulnerabilities

### Container Security
- No secrets in images or env vars (use secrets management)
- Read-only filesystem where possible
- Drop all capabilities, add only needed ones
- Resource limits (CPU, memory)

## Kubernetes Patterns

### Deployment Strategy
| Strategy | When to Use | Risk |
|----------|-------------|------|
| Rolling Update | Default, zero-downtime | Slow rollback |
| Blue-Green | Instant switch | Double resources |
| Canary | Gradual rollout | Complex routing |
| Recreate | Downtime OK | Simple |

### Health Probes
```yaml
livenessProbe:
  httpGet:
    path: /healthz
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 3
  periodSeconds: 5
```

### Resource Management
```yaml
resources:
  requests:
    memory: "128Mi"
    cpu: "250m"
  limits:
    memory: "256Mi"
    cpu: "500m"
```

## Release Management

### Versioning
- **Semantic Versioning**: MAJOR.MINOR.PATCH
- **Pre-release**: 1.0.0-alpha.1, 1.0.0-beta.1
- **Build metadata**: 1.0.0+build.123

### Rollback Strategy
1. Automated rollback on health check failure
2. Manual rollback trigger (< 5 min)
3. Database migration compatibility
4. Feature flags for instant disable

### Release Checklist
- [ ] All tests passing
- [ ] Security scan clean
- [ ] Performance benchmarks acceptable
- [ ] Changelog updated
- [ ] Version bumped
- [ ] Rollback plan documented

## Infrastructure as Code

### Principles
- Everything in version control
- Immutable infrastructure
- Declarative configuration
- Reproducible environments

### Tools
| Category | Tool |
|----------|------|
| Provisioning | Terraform, Pulumi |
| Configuration | Ansible, Chef |
| Containers | Docker, Podman |
| Orchestration | Kubernetes, Nomad |
| Service Mesh | Istio, Linkerd |

## Integration Points

- **architecture-auditor**: Deployment health in audit reports
- **Observability**: Pipeline metrics, deployment tracking
- **Testing**: E2E tests in staging before prod
- **Performance**: Benchmark gates in CI
