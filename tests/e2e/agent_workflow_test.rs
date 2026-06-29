#[cfg(feature = "integration_tests")]
mod agent_workflow_e2e {
    use std::sync::Once;

    fn check_env() {
        static CHECKED: Once = Once::new();
        CHECKED.call_once(|| {
            if std::env::var("NEOTRIX_TEST").is_err() {
                panic!(
                    "E2E tests require NEOTRIX_TEST env var.\n\
                     Set it via: NEOTRIX_TEST=1 cargo test --features integration_tests -p neotrix"
                );
            }
        });
    }

    #[test]
    fn agent_workflow_creation() {
        check_env();
        use neotrix::agent::agent_workflow::{AgentWorkflow, PlanMode};

        let wf = AgentWorkflow::new("test-1", "Test Workflow", "Do something");
        assert_eq!(wf.id, "test-1");
        assert_eq!(wf.display_name, "Test Workflow");
        assert_eq!(wf.instructions, "Do something");
        assert_eq!(wf.plan_mode, PlanMode::Execute);
        assert!(wf.steps.is_empty());
    }

    #[test]
    fn agent_workflow_new_plan() {
        check_env();
        use neotrix::agent::agent_workflow::{AgentWorkflow, PlanMode};

        let wf = AgentWorkflow::new_plan("plan-1", "Plan Mode Test", "Explore first");
        assert_eq!(wf.plan_mode, PlanMode::Explore);
    }

    #[test]
    fn agent_workflow_add_and_execute_steps() {
        check_env();
        use neotrix::agent::agent_workflow::{AgentStep, AgentWorkflow};

        let mut wf = AgentWorkflow::new("steps-1", "Step Test", "Execute steps");
        wf.add_step(AgentStep::Think {
            thought: "step one".to_string(),
        });
        wf.add_step(AgentStep::Think {
            thought: "step two".to_string(),
        });
        wf.add_step(AgentStep::EndTurn {
            result: Some("done".to_string()),
        });

        assert_eq!(wf.steps.len(), 3);

        let result = wf.execute();
        assert!(result.success, "think steps should succeed");
        assert_eq!(result.steps_executed, 3);
        assert_eq!(result.final_output, Some("done".to_string()));
    }

    #[test]
    fn agent_workflow_plan_mode_blocks_mutation() {
        check_env();
        use neotrix::agent::agent_workflow::{AgentStep, AgentWorkflow};

        let mut wf =
            AgentWorkflow::new_plan("plan-block-1", "Plan Block Test", "Should block writes");
        wf.add_step(AgentStep::ReadFile {
            path: "/tmp/test.txt".to_string(),
        });
        wf.add_step(AgentStep::EditFile {
            path: "/tmp/test.txt".to_string(),
            content: "modified".to_string(),
        });

        let result = wf.execute();
        assert!(!result.success, "explore mode should block mutation steps");
        assert_eq!(
            result.steps_executed, 2,
            "should process read then block on edit"
        );
        assert!(
            result.final_output.as_ref().unwrap().contains("blocked"),
            "output should mention blocked"
        );
    }

    #[test]
    fn agent_workflow_plan_mode_allows_read_only() {
        check_env();
        use neotrix::agent::agent_workflow::{AgentStep, AgentWorkflow};

        let mut wf = AgentWorkflow::new_plan("plan-read-1", "Plan Read Test", "Should allow reads");
        wf.add_step(AgentStep::ReadFile {
            path: "Cargo.toml".to_string(),
        });
        wf.add_step(AgentStep::Think {
            thought: "analysis".to_string(),
        });

        let result = wf.execute();
        assert!(result.success, "explore mode should allow read/think steps");
    }
}
