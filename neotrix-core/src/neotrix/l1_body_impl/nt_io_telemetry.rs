use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use tracing::Span;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Registry;

pub fn init_otel(service_name: &str) -> bool {
    let endpoint = match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Ok(v) if !v.is_empty() => v,
        _ => return false,
    };

    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&endpoint)
        .build()
    {
        Ok(e) => e,
        Err(e) => {
            log::warn!("OTel exporter build failed (non-fatal): {}", e);
            return false;
        }
    };

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(
            Resource::new(vec![KeyValue::new("service.name", service_name.to_string())]),
        )
        .build();

    let tracer = provider.tracer("neotrix");
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    Registry::default().with(telemetry).init();

    true
}

pub fn agent_span(agent_id: &str, task: &str) -> Span {
    tracing::info_span!(
        "agent",
        agent.id = %agent_id,
        task = %task,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_span_creates_span() {
        let span = agent_span("test-agent", "test-task");
        assert_eq!(span.metadata().map(|m| m.name()), Some("agent"));
    }

    #[test]
    fn test_init_otel_no_endpoint() {
        let result = init_otel("neotrix-test");
        assert!(!result);
    }
}
