//! 流程引擎测试
//!
//! 测试 process_engine.rs 中定义的流程引擎核心功能。
//! 覆盖状态枚举、流程定义、条件评估、实例管理等。

#![forbid(unsafe_code)]

use super::super::process_engine::*;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHandler;

    impl StepHandler for MockHandler {
        fn execute(
            &self,
            _instance: &ProcessInstance,
            _step: &ProcessStep,
            context: &mut HashMap<String, String>,
        ) -> Result<HashMap<String, String>, String> {
            let mut output = HashMap::new();
            output.insert("result".to_string(), "success".to_string());
            context.insert("mock_done".to_string(), "true".to_string());
            Ok(output)
        }
    }

    struct FailingHandler;

    impl StepHandler for FailingHandler {
        fn execute(
            &self,
            _instance: &ProcessInstance,
            _step: &ProcessStep,
            _context: &mut HashMap<String, String>,
        ) -> Result<HashMap<String, String>, String> {
            Err("Intentional failure".to_string())
        }
    }

    #[test]
    fn test_process_status_display() {
        assert_eq!(ProcessStatus::Pending.to_string(), "Pending");
        assert_eq!(ProcessStatus::Running.to_string(), "Running");
        assert_eq!(ProcessStatus::Paused.to_string(), "Paused");
        assert_eq!(ProcessStatus::Completed.to_string(), "Completed");
        assert_eq!(ProcessStatus::Failed.to_string(), "Failed");
        assert_eq!(ProcessStatus::Cancelled.to_string(), "Cancelled");
    }

    #[test]
    fn test_step_status_is_terminal() {
        assert!(StepStatus::Completed.is_terminal());
        assert!(StepStatus::Failed.is_terminal());
        assert!(StepStatus::Skipped.is_terminal());
        assert!(!StepStatus::Pending.is_terminal());
        assert!(!StepStatus::Running.is_terminal());
    }

    #[test]
    fn test_process_status_is_terminal() {
        assert!(ProcessStatus::Completed.is_terminal());
        assert!(ProcessStatus::Failed.is_terminal());
        assert!(ProcessStatus::Cancelled.is_terminal());
        assert!(!ProcessStatus::Pending.is_terminal());
        assert!(!ProcessStatus::Running.is_terminal());
        assert!(!ProcessStatus::Paused.is_terminal());
    }

    #[test]
    fn test_condition_always() {
        let ctx = HashMap::new();
        assert!(evaluate_condition(&StepCondition::Always, &ctx));
    }

    #[test]
    fn test_condition_equals() {
        let mut ctx = HashMap::new();
        ctx.insert("status".to_string(), "active".to_string());

        assert!(evaluate_condition(
            &StepCondition::Equals {
                key: "status".to_string(),
                value: "active".to_string(),
            },
            &ctx
        ));

        assert!(!evaluate_condition(
            &StepCondition::Equals {
                key: "status".to_string(),
                value: "inactive".to_string(),
            },
            &ctx
        ));

        assert!(!evaluate_condition(
            &StepCondition::Equals {
                key: "missing".to_string(),
                value: "any".to_string(),
            },
            &ctx
        ));
    }

    #[test]
    fn test_condition_key_exists() {
        let mut ctx = HashMap::new();
        ctx.insert("key1".to_string(), "value1".to_string());

        assert!(evaluate_condition(
            &StepCondition::KeyExists {
                key: "key1".to_string(),
            },
            &ctx
        ));

        assert!(!evaluate_condition(
            &StepCondition::KeyExists {
                key: "missing".to_string(),
            },
            &ctx
        ));
    }

    #[test]
    fn test_condition_custom() {
        let ctx = HashMap::new();
        // Custom conditions always return true (handled by handler.validate)
        assert!(evaluate_condition(
            &StepCondition::Custom("custom_check".to_string()),
            &ctx
        ));
    }

    #[test]
    fn test_process_definition_find_step() {
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![
                ProcessStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        };

        assert!(def.find_step("step1").is_some());
        assert!(def.find_step("step2").is_some());
        assert!(def.find_step("nonexistent").is_none());
    }

    #[test]
    fn test_process_definition_step_index() {
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![
                ProcessStep {
                    id: "a".to_string(),
                    name: "A".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "b".to_string(),
                    name: "B".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        };

        assert_eq!(def.step_index("a"), Some(0));
        assert_eq!(def.step_index("b"), Some(1));
        assert_eq!(def.step_index("c"), None);
    }

    #[test]
    fn test_engine_create_instance() {
        let mut engine = ProcessEngine::new();
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let mut ctx = HashMap::new();
        ctx.insert("key".to_string(), "value".to_string());

        let result = engine.create_instance("test", ctx);
        assert!(result.is_ok());

        let inst_id = result.unwrap();
        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Pending);
        assert_eq!(inst.context.get("key").unwrap(), "value");
    }

    #[test]
    fn test_engine_create_instance_unknown_def() {
        let mut engine = ProcessEngine::new();
        let result = engine.create_instance("nonexistent", HashMap::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_start_instance() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        let result = engine.start_instance(&inst_id);
        assert!(result.is_ok());

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Completed);
        assert_eq!(inst.step_results.len(), 1);
        assert_eq!(inst.step_results[0].status, StepStatus::Completed);
    }

    #[test]
    fn test_engine_start_non_pending() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        // Try to start again (now completed)
        let result = engine.start_instance(&inst_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_cancel_instance() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        let result = engine.cancel_instance(&inst_id);
        assert!(result.is_ok());

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Cancelled);
    }

    #[test]
    fn test_engine_cancel_terminal() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        // Try to cancel completed instance
        let result = engine.cancel_instance(&inst_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_pause_resume() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        // Pause
        let result = engine.pause_instance(&inst_id);
        assert!(result.is_ok());
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Paused
        );

        // Resume
        let result = engine.resume_instance(&inst_id);
        assert!(result.is_ok());
        assert_eq!(
            engine.get_instance(&inst_id).unwrap().status,
            ProcessStatus::Completed
        );
    }

    #[test]
    fn test_engine_pause_non_running() {
        let mut engine = ProcessEngine::new();
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        let result = engine.pause_instance(&inst_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_resume_non_paused() {
        let mut engine = ProcessEngine::new();
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        let result = engine.resume_instance(&inst_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_engine_step_with_retry() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 3,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.step_results[0].retries, 3);
    }

    #[test]
    fn test_engine_step_failure() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("failing", Arc::new(FailingHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "failing".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Failed);
        assert_eq!(inst.step_results[0].status, StepStatus::Failed);
    }

    #[test]
    fn test_engine_conditional_step_skip() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![
                ProcessStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Equals {
                        key: "run_step2".to_string(),
                        value: "true".to_string(),
                    },
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        };
        engine.register_definition(def);

        let mut ctx = HashMap::new();
        ctx.insert("run_step2".to_string(), "false".to_string());

        let inst_id = engine.create_instance("test", ctx).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let inst = engine.get_instance(&inst_id).unwrap();
        assert_eq!(inst.status, ProcessStatus::Completed);
        assert_eq!(inst.step_results[0].status, StepStatus::Completed);
        assert_eq!(inst.step_results[1].status, StepStatus::Skipped);
    }

    #[test]
    fn test_engine_list_instances() {
        let mut engine = ProcessEngine::new();
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        engine.create_instance("test", HashMap::new()).unwrap();
        engine.create_instance("test", HashMap::new()).unwrap();
        engine.create_instance("test", HashMap::new()).unwrap();

        assert_eq!(engine.list_instances().len(), 3);
    }

    #[test]
    fn test_engine_list_instances_by_status() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let _ = engine.create_instance("test", HashMap::new());
        let _ = engine.create_instance("test", HashMap::new());
        let inst3 = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst3).unwrap();

        let pending = engine.list_instances_by_status(ProcessStatus::Pending);
        let completed = engine.list_instances_by_status(ProcessStatus::Completed);

        assert_eq!(pending.len(), 2);
        assert_eq!(completed.len(), 1);
    }

    #[test]
    fn test_engine_get_progress() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![
                ProcessStep {
                    id: "step1".to_string(),
                    name: "Step 1".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
                ProcessStep {
                    id: "step2".to_string(),
                    name: "Step 2".to_string(),
                    description: "".to_string(),
                    handler: "mock".to_string(),
                    condition: StepCondition::Always,
                    timeout_ms: 0,
                    max_retries: 0,
                    params: HashMap::new(),
                },
            ],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        assert_eq!(engine.get_progress(&inst_id), Some(0.0));

        engine.start_instance(&inst_id).unwrap();
        assert_eq!(engine.get_progress(&inst_id), Some(1.0));
    }

    #[test]
    fn test_engine_get_total_duration() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![ProcessStep {
                id: "step1".to_string(),
                name: "Step 1".to_string(),
                description: "".to_string(),
                handler: "mock".to_string(),
                condition: StepCondition::Always,
                timeout_ms: 0,
                max_retries: 0,
                params: HashMap::new(),
            }],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        assert_eq!(engine.get_total_duration_ms(&inst_id), Some(0));

        engine.start_instance(&inst_id).unwrap();
        let duration = engine.get_total_duration_ms(&inst_id).unwrap();
        assert!(duration >= 0);
    }

    #[test]
    fn test_engine_delete_instance() {
        let mut engine = ProcessEngine::new();
        engine.register_handler("mock", Arc::new(MockHandler));

        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        engine.start_instance(&inst_id).unwrap();

        let result = engine.delete_instance(&inst_id);
        assert!(result.is_ok());
        assert!(engine.get_instance(&inst_id).is_none());
    }

    #[test]
    fn test_engine_delete_non_terminal() {
        let mut engine = ProcessEngine::new();
        let def = ProcessDefinition {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            steps: vec![],
        };
        engine.register_definition(def);

        let inst_id = engine.create_instance("test", HashMap::new()).unwrap();
        let result = engine.delete_instance(&inst_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_op_handler() {
        let handler = NoOpHandler;
        let instance = ProcessInstance {
            id: "test".to_string(),
            definition_id: "test".to_string(),
            status: ProcessStatus::Pending,
            current_step_index: 0,
            context: HashMap::new(),
            step_results: vec![],
            created_at: 0,
            updated_at: 0,
        };
        let step = ProcessStep {
            id: "step".to_string(),
            name: "Step".to_string(),
            description: "".to_string(),
            handler: "noop".to_string(),
            condition: StepCondition::Always,
            timeout_ms: 0,
            max_retries: 0,
            params: HashMap::new(),
        };
        let mut context = HashMap::new();

        let result = handler.execute(&instance, &step, &mut context);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
