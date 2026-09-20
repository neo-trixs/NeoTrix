---
name: observability
description: "Observability stack — structured logging, metrics collection, distributed tracing, and alerting"
version: "1.0.0"
author: "NeoTrix"
triggers: "observ|log|metric|trac|alert|monitor|telemetry|instrument|span|dashboard"
---

# Observability — Logging, Metrics, Tracing

The three pillars of observability: logs, metrics, and traces. Know what your system is doing and why.

## Three Pillars

### Structured Logging
- **What happened** at a specific point in time
- **Tool**: `tracing` crate (Rust), `tracing-subscriber`
- **Format**: JSON lines, structured fields

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(db), fields(user_id = %user.id))]
async fn process_order(db: &Db, user: &User, order: &Order) {
    info!(order_id = %order.id, total = order.total, "processing order");
    // ...
    warn!(retry_count = 3, "retrying database connection");
}
```

### Metrics
- **Quantitative measurements** over time
- **Types**: Counter, Gauge, Histogram
- **Tool**: `metrics` crate, Prometheus, Grafana

```rust
use metrics::{counter, histogram, describe_counter};

counter!("orders.processed", "status" => "success").increment(1);
histogram!("order.processing.duration_ms").record(elapsed.as_millis() as f64);
```

### Distributed Tracing
- **Request flow** across service boundaries
- **Tool**: `tracing-opentelemetry`, Jaeger, Zipkin
- **Concept**: Spans, contexts, baggage

```rust
use tracing_opentelemetry::OpenTelemetrySpanExt;

let span = info_span!("http_request", method = %req.method(), path = %req.path());
let _guard = span.enter();
// All logs/metrics within inherit this span's context
```

## Alert Design

| Severity | Response Time | Example |
|----------|---------------|---------|
| P1 Critical | Immediate | Service down, data loss |
| P2 High | < 1 hour | Degraded performance |
| P3 Medium | < 4 hours | Non-critical failures |
| P4 Low | Next day | Warnings, anomalies |

### Alert Anti-Patterns
- Alerting on symptoms instead of causes
- Too many alerts (alert fatigue)
- No runbooks attached
- Alerts that require manual investigation

## Dashboard Design

### Golden Signals (Google SRE)
1. **Latency**: Time to serve a request
2. **Traffic**: Requests per second
3. **Errors**: Error rate
4. **Saturation**: How full is the service

### RED Method (Microservices)
- **Rate**: Requests per second
- **Errors**: Errors per second
- **Duration**: Latency distribution

### USE Method (Resources)
- **Utilization**: % of resource used
- **Saturation**: Queue depth
- **Errors**: Error count

## Instrumentation Checklist

- [ ] All HTTP endpoints have request/response metrics
- [ ] Database queries have duration histograms
- [ ] External calls have latency and error tracking
- [ ] Business events have structured logs
- [ ] Critical paths have distributed traces
- [ ] Resource usage has gauges

## Integration Points

- **architecture-auditor**: Observability coverage in audit (logging completeness)
- **Performance**: Metrics validate optimization impact
- **Deployment**: Observability stack deployed with services
- **Testing**: Observability in test environments
