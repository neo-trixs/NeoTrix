---
name: documentation
description: "Documentation engineering — API docs, architecture decision records, runbooks, and knowledge management"
version: "1.0.0"
author: "NeoTrix"
triggers: "doc|document|readme|adr|architecture decision|runbook|api doc|swagger|openapi|changelog|rfc"
---

# Documentation — API Docs, Architecture Docs, Runbooks

Documentation as code. Keep docs close to code, version-controlled, and automatically validated.

## Documentation Types

### 1. API Documentation
- **Format**: OpenAPI/Swagger, AsyncAPI, GraphQL schema
- **Tool**: `utoipa` (Rust), `swagger-ui`, `redoc`
- **Location**: `docs/api/` or inline with code

```rust
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(("id" = i64, Path, description = "User ID")),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn get_user(Path(id): Path<i64>) -> Json<User> { ... }
```

### 2. Architecture Decision Records (ADRs)
- **Purpose**: Document why decisions were made
- **Format**: Markdown in `docs/adr/`
- **Lifecycle**: Proposed → Accepted → Deprecated/Superseded

```markdown
# ADR-001: Use Event-Driven Architecture

## Status
Accepted

## Context
System needs loose coupling between components...

## Decision
Adopt event-driven architecture with EventBus pattern.

## Consequences
+ Loose coupling
+ Async by default
- Eventual consistency
- Debugging complexity
```

### 3. Runbooks
- **Purpose**: Step-by-step operational guides
- **Audience**: On-call engineers
- **Format**: Markdown with clear steps

```markdown
# Runbook: High Latency Alert

## Symptoms
- P99 latency > 500ms
- Error rate elevated

## Diagnosis
1. Check `kubectl top pods` for resource pressure
2. Review recent deployments
3. Check database connection pool metrics

## Remediation
1. Scale up: `kubectl scale deployment api --replicas=5`
2. Rollback if recent deploy: `kubectl rollout undo deployment/api`
```

### 4. Architecture Documentation
- **C4 Model**: Context, Container, Component, Code
- **Diagrams**: Mermaid, PlantUML, structurizr
- **Location**: `docs/architecture/`

### 5. Code Documentation
- **Rust**: `///` doc comments, `//!` module docs
- **Coverage**: Public APIs must have docs
- **Examples**: Include runnable examples

## Documentation Standards

### Writing Style
- Use active voice
- Keep sentences short
- Include code examples
- Version-stamp API docs

### Structure
```
docs/
├── README.md              # Project overview
├── api/                   # API documentation
│   └── openapi.yaml
├── architecture/          # Architecture docs
│   ├── context.md
│   └── decisions/
│       └── ADR-001.md
├── guides/                # How-to guides
│   ├── getting-started.md
│   └── deployment.md
├── operations/            # Runbooks
│   ├── alert-response.md
│   └── troubleshooting.md
└── CHANGELOG.md           # Release notes
```

### Validation
- Link checking in CI
- Code example compilation
- OpenAPI schema validation
- Spell checking

## Changelog Management

### Keep a Changelog Format
```markdown
# Changelog

## [1.2.0] - 2026-09-19
### Added
- New feature X

### Changed
- Improved Y performance

### Fixed
- Bug in Z

### Deprecated
- Feature W (use V instead)

### Removed
- Feature U

### Security
- Vulnerability fix
```

## Integration Points

- **architecture-auditor**: ADR tracking, doc completeness metrics
- **Deployment**: Runbooks for operational procedures
- **Testing**: API docs auto-generated from code
- **Observability**: Dashboard documentation, alert runbooks
