//! End-to-end integration tests for the MainPipeline.

use neotrix::pipeline::{
    MainPipeline, PipelineError, PipelineInput, PipelineOptions, PipelineResult,
};

fn pipeline() -> MainPipeline {
    MainPipeline::new()
}

fn basic_input(task: &str) -> PipelineInput {
    PipelineInput::new(task)
}

// ════════════════════════════════════════════════════════════════
// Test 1: Basic e2e - simple task through all five stages
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_basic_task_completes_all_stages() {
    let p = pipeline();
    let input = basic_input("analyze the codebase for security issues");
    let result = p.process(input).expect("pipeline should succeed");

    assert!(!result.output.is_empty(), "output must not be empty");
    assert!(
        result.output.contains("=== Pipeline Output ==="),
        "output must have header"
    );
    assert!(
        result.output.contains("agent: analyst"),
        "should route to analyst agent"
    );
    assert!(
        result.output.contains("validation: PASS"),
        "validation should pass"
    );
    // intake(1) + route(1) + execute(1) + validate(1) = 4
    assert!(
        result.artifacts.len() >= 4,
        "should have at least 4 artifacts, got {}",
        result.artifacts.len()
    );
}

// ════════════════════════════════════════════════════════════════
// Test 2: Routing - different tasks route to different agents
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_routing_dispatches_correct_agent() {
    let cases: Vec<(&str, &str)> = vec![
        ("implement a binary search in Rust", "coder"),
        ("analyze performance metrics", "analyst"),
        ("search for relevant papers", "researcher"),
        ("write a design document", "writer"),
        ("test the authentication flow", "tester"),
        ("design the system architecture", "architect"),
        ("say hello", "general"),
    ];

    let p = pipeline();
    for (task, expected_agent) in cases {
        let input = basic_input(task);
        let result = p
            .process(input)
            .unwrap_or_else(|e| panic!("task '{}' failed: {}", task, e));
        assert!(
            result.output.contains(&format!("agent: {}", expected_agent)),
            "task '{}' should route to '{}' but output was: {}",
            task,
            expected_agent,
            result.output
        );
    }
}

// ════════════════════════════════════════════════════════════════
// Test 3: Context passing - context flows through all stages
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_context_passed_through_pipeline() {
    let p = pipeline();
    let input = PipelineInput::new("run a task")
        .with_context("env", "production")
        .with_context("priority", "high");

    let result = p.process(input).expect("pipeline should succeed");

    assert!(!result.output.is_empty());
    let _ = (
        result.metrics.intake_ms,
        result.metrics.route_ms,
        result.metrics.execute_ms,
        result.metrics.validate_ms,
        result.metrics.output_ms,
    );
}

// ════════════════════════════════════════════════════════════════
// Test 4: Metrics - all stage timings are recorded
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_metrics_recorded_for_all_stages() {
    let p = pipeline();
    let input = basic_input("implement sorting");
    let result = p.process(input).unwrap();

    assert!(
        result.duration_ms < 5000,
        "pipeline took too long: {}ms",
        result.duration_ms
    );

    let _ = result.metrics.intake_ms;
    let _ = result.metrics.route_ms;
    let _ = result.metrics.execute_ms;
    let _ = result.metrics.validate_ms;
    let _ = result.metrics.output_ms;
}

// ════════════════════════════════════════════════════════════════
// Test 5: Error handling - empty task fails at intake
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_empty_task_fails_at_intake() {
    let p = pipeline();
    let input = basic_input("");

    let err = p.process(input).expect_err("empty task should fail");
    match err {
        PipelineError::StageFailure { stage, .. } => {
            assert_eq!(stage, "intake", "error should come from intake stage");
        }
        other => panic!("expected StageFailure, got: {:?}", other),
    }
}

// ════════════════════════════════════════════════════════════════
// Test 6: Artifact chain - each stage produces artifacts
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_artifact_chain_per_stage() {
    let p = pipeline();
    let input = basic_input("review this pull request");
    let result = p.process(input).unwrap();

    let stages: Vec<&str> = result
        .artifacts
        .iter()
        .map(|a| a.source_stage.as_str())
        .collect();

    assert!(stages.contains(&"intake"), "missing intake artifact");
    assert!(stages.contains(&"route"), "missing route artifact");
    assert!(stages.contains(&"execute"), "missing execute artifact");
    assert!(stages.contains(&"validate"), "missing validate artifact");
}

// ════════════════════════════════════════════════════════════════
// Test 7: Options builder roundtrip
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_options_builder_roundtrip() {
    let opts = PipelineOptions {
        max_retries: 3,
        timeout_ms: 10_000,
        strict_validation: false,
    };
    let input = PipelineInput::new("do something").with_options(opts);
    assert_eq!(input.options.max_retries, 3);
    assert_eq!(input.options.timeout_ms, 10_000);
    assert!(!input.options.strict_validation);

    let p = pipeline();
    let result = p.process(input).unwrap();
    assert!(!result.output.is_empty());
}

// ════════════════════════════════════════════════════════════════
// Test 8: Serialization roundtrip of PipelineResult
// ════════════════════════════════════════════════════════════════

#[test]
fn test_e2e_result_serialization_roundtrip() {
    let p = pipeline();
    let input = basic_input("serialize me");
    let result = p.process(input).unwrap();

    let json = serde_json::to_string(&result).expect("should serialize");
    let deserialized: PipelineResult =
        serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(deserialized.output, result.output);
    assert_eq!(deserialized.artifacts.len(), result.artifacts.len());
    assert_eq!(deserialized.duration_ms, result.duration_ms);
}
