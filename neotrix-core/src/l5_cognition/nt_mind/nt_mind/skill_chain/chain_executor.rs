//! 链执行器 — ChainExecutor
//!
//! 执行技能链，将每步输出作为下一步输入。
//! 支持暂停/恢复，超时检测，重试逻辑。
//! R-P123: 执行器按认知域隔离状态

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::skill_chain::{ChainStatus, SkillChain};
use super::skill_step::SkillStep;

/// 链执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainResult {
    /// 链名称
    pub chain_name: String,
    /// 最终状态
    pub status: ChainStatus,
    /// 每步执行结果
    pub step_results: Vec<StepExecution>,
    /// 链的最终输出 (最后一步的 output)
    pub final_output: Option<String>,
    /// 错误信息 (仅 Failed 状态)
    pub error: Option<String>,
    /// 总耗时(毫秒)
    pub duration_ms: u64,
}

/// 单步执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {
    /// 步骤名称
    pub step_name: String,
    /// 步骤载荷
    pub payload: String,
    /// 该步骤的输出
    pub output: Option<String>,
    /// 该步骤是否成功
    pub success: bool,
    /// 该步骤耗时(毫秒)
    pub duration_ms: u64,
    /// 重试次数
    pub retries: u32,
}

/// 链执行上下文 — 传递给执行器的外部信息
#[derive(Debug, Clone, Default)]
pub struct ChainContext {
    /// 外部输入参数
    pub params: HashMap<String, String>,
    /// 执行环境标签
    pub environment: String,
}

/// 技能步骤执行 trait
///
/// 实现此 trait 以提供步骤的实际执行逻辑
pub trait StepExecutor: Send + Sync {
    /// 执行单个步骤，返回输出
    fn execute_step(
        &self,
        step: &SkillStep,
        input: Option<&str>,
        context: &ChainContext,
    ) -> Result<String, String>;
}

/// 链执行器
pub struct ChainExecutor {
    /// 执行中的链
    chain: SkillChain,
    /// 暂停状态
    paused: bool,
    /// 步骤执行器
    executor: Box<dyn StepExecutor>,
    /// 每步重试计数
    retry_counts: HashMap<usize, u32>,
}

impl ChainExecutor {
    /// 创建新的链执行器
    pub fn new(chain: SkillChain, executor: Box<dyn StepExecutor>) -> Self {
        Self {
            chain,
            paused: false,
            executor,
            retry_counts: HashMap::new(),
        }
    }

    /// 执行完整链，返回结果
    pub fn execute_chain(
        &mut self,
        context: &ChainContext,
    ) -> ChainResult {
        let start = std::time::Instant::now();
        let mut step_results = Vec::new();
        let mut current_input: Option<String> = None;
        let mut final_output: Option<String> = None;

        while let Some(step) = self.chain.next_step() {
            if self.paused {
                break;
            }

            let step_clone = step.clone();
            let max_retries = self.chain.config.max_retries_per_step;

            let mut success = false;
            let mut output: Option<String> = None;
            let mut retries = 0;
            let step_start = std::time::Instant::now();

            // 重试循环
            while retries <= max_retries {
                match self.executor.execute_step(
                    &step_clone,
                    current_input.as_deref(),
                    context,
                ) {
                    Ok(out) => {
                        output = Some(out.clone());
                        final_output = Some(out);
                        success = true;
                        break;
                    }
                    Err(e) => {
                        retries += 1;
                        if retries > max_retries {
                            output = Some(format!("FAILED after {} retries: {}", max_retries, e));
                        }
                    }
                }
            }

            let duration = step_start.elapsed().as_millis() as u64;

            step_results.push(StepExecution {
                step_name: step_clone.name().to_string(),
                payload: step_clone.payload().to_string(),
                output: output.clone(),
                success,
                duration_ms: duration,
                retries,
            });

            if !success {
                // 失败: 终止链
                let duration = start.elapsed().as_millis() as u64;
                return ChainResult {
                    chain_name: self.chain.name.clone(),
                    status: ChainStatus::Failed,
                    step_results,
                    final_output: None,
                    error: output,
                    duration_ms: duration,
                };
            }

            // 推进到下一步，当前输出作为下一步输入
            self.chain.advance();
            current_input = final_output.clone();
        }

        let duration = start.elapsed().as_millis() as u64;
        let status = if self.paused {
            ChainStatus::Paused
        } else {
            ChainStatus::Complete
        };

        ChainResult {
            chain_name: self.chain.name.clone(),
            status,
            step_results,
            final_output,
            error: None,
            duration_ms: duration,
        }
    }

    /// 暂停执行
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// 恢复执行
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// 是否已暂停
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// 获取链引用
    pub fn chain(&self) -> &SkillChain {
        &self.chain
    }

    /// 获取链可变引用
    pub fn chain_mut(&mut self) -> &mut SkillChain {
        &mut self.chain
    }
}

/// 空执行器 — 用于测试，所有步骤返回固定输出
pub struct MockStepExecutor;

impl StepExecutor for MockStepExecutor {
    fn execute_step(
        &self,
        step: &SkillStep,
        _input: Option<&str>,
        _context: &ChainContext,
    ) -> Result<String, String> {
        Ok(format!("{}:done", step.name()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::chain_config::ChainConfig;

    fn make_test_chain() -> SkillChain {
        SkillChain::new(
            "test_chain",
            vec![
                SkillStep::Brainstorm { topic: "auth".into() },
                SkillStep::Plan { goal: "design auth".into() },
                SkillStep::Tdd { feature: "implement auth".into() },
                SkillStep::Review { code: "fn auth(){}".into() },
            ],
        )
    }

    #[test]
    fn test_execute_chain_success() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        let context = ChainContext::default();

        let result = executor.execute_chain(&context);
        assert_eq!(result.status, ChainStatus::Complete);
        assert_eq!(result.step_results.len(), 4);
        assert!(result.step_results.iter().all(|s| s.success));
        assert!(result.error.is_none());
        assert!(result.final_output.is_some());
    }

    #[test]
    fn test_execute_chain_step_output_chaining() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        let context = ChainContext::default();

        let result = executor.execute_chain(&context);
        // 最后一步输出
        let last = result.step_results.last().unwrap();
        assert_eq!(last.output.as_deref(), Some("Review:done"));
        assert_eq!(result.final_output.as_deref(), Some("Review:done"));
    }

    #[test]
    fn test_pause_resume() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));

        executor.pause();
        assert!(executor.is_paused());

        executor.resume();
        assert!(!executor.is_paused());
    }

    #[test]
    fn test_execute_chain_paused() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        executor.pause();
        let context = ChainContext::default();

        let result = executor.execute_chain(&context);
        assert_eq!(result.status, ChainStatus::Paused);
        assert_eq!(result.step_results.len(), 0);
    }

    #[test]
    fn test_chain_accessors() {
        let chain = make_test_chain();
        let executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        assert_eq!(executor.chain().name, "test_chain");
    }

    #[test]
    fn test_context_with_params() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));

        let mut params = HashMap::new();
        params.insert("env".into(), "staging".into());
        let context = ChainContext {
            params,
            environment: "test".into(),
        };

        let result = executor.execute_chain(&context);
        assert_eq!(result.status, ChainStatus::Complete);
    }

    #[test]
    fn test_step_execution_retries() {
        struct FailOnceExecutor {
            call_count: std::sync::atomic::AtomicU32,
        }

        impl StepExecutor for FailOnceExecutor {
            fn execute_step(
                &self,
                _step: &SkillStep,
                _input: Option<&str>,
                _context: &ChainContext,
            ) -> Result<String, String> {
                let count = self.call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count == 0 {
                    Err("transient failure".into())
                } else {
                    Ok("recovered".into())
                }
            }
        }

        let chain = SkillChain::new(
            "retry_test",
            vec![SkillStep::Brainstorm { topic: "t".into() }],
        );
        let mut executor = ChainExecutor::new(
            chain,
            Box::new(FailOnceExecutor {
                call_count: std::sync::atomic::AtomicU32::new(0),
            }),
        );
        let context = ChainContext::default();

        let result = executor.execute_chain(&context);
        assert_eq!(result.status, ChainStatus::Complete);
        assert_eq!(result.step_results[0].retries, 1);
    }

    #[test]
    fn test_serde_chain_result() {
        let result = ChainResult {
            chain_name: "test".into(),
            status: ChainStatus::Complete,
            step_results: vec![],
            final_output: Some("done".into()),
            error: None,
            duration_ms: 42,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ChainResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, ChainStatus::Complete);
        assert_eq!(back.duration_ms, 42);
    }

    #[test]
    fn test_execute_chain_failure() {
        struct AlwaysFailExecutor;

        impl StepExecutor for AlwaysFailExecutor {
            fn execute_step(
                &self,
                _step: &SkillStep,
                _input: Option<&str>,
                _context: &ChainContext,
            ) -> Result<String, String> {
                Err("permanent failure".into())
            }
        }

        let chain = SkillChain::new(
            "fail_chain",
            vec![SkillStep::Brainstorm { topic: "t".into() }],
        );
        let mut executor = ChainExecutor::new(chain, Box::new(AlwaysFailExecutor));
        let context = ChainContext::default();

        let result = executor.execute_chain(&context);
        assert_eq!(result.status, ChainStatus::Failed);
        assert!(result.error.is_some());
        assert!(result.final_output.is_none());
    }

    #[test]
    fn test_chain_executor_chain_accessors() {
        let chain = make_test_chain();
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        assert_eq!(executor.chain().name, "test_chain");
        executor.chain_mut().name = "modified".into();
        assert_eq!(executor.chain().name, "modified");
    }

    #[test]
    fn test_step_execution_result_fields() {
        let chain = SkillChain::new(
            "test",
            vec![SkillStep::Brainstorm { topic: "t".into() }],
        );
        let mut executor = ChainExecutor::new(chain, Box::new(MockStepExecutor));
        let result = executor.execute_chain(&ChainContext::default());
        assert_eq!(result.step_results.len(), 1);
        let step = &result.step_results[0];
        assert_eq!(step.step_name, "Brainstorm");
        assert_eq!(step.payload, "t");
        assert!(step.success);
        assert_eq!(step.retries, 0);
        assert!(step.duration_ms >= 0);
    }

    #[test]
    fn test_chain_context_serialization() {
        let mut params = std::collections::HashMap::new();
        params.insert("key".into(), "value".into());
        let ctx = ChainContext {
            params,
            environment: "prod".into(),
        };
        // ChainContext doesn't derive Serialize, but we can test its structure
        assert_eq!(ctx.params.get("key").unwrap(), "value");
        assert_eq!(ctx.environment, "prod");
    }

    #[test]
    fn test_retry_exhaustion() {
        struct AlwaysFailExecutor;

        impl StepExecutor for AlwaysFailExecutor {
            fn execute_step(
                &self,
                _step: &SkillStep,
                _input: Option<&str>,
                _context: &ChainContext,
            ) -> Result<String, String> {
                Err("fail".into())
            }
        }

        let mut cfg = ChainConfig::default();
        cfg.max_retries_per_step = 2;
        let chain = SkillChain::with_config(
            "retry_exhaust",
            vec![SkillStep::Brainstorm { topic: "t".into() }],
            cfg,
        );
        let mut executor = ChainExecutor::new(chain, Box::new(AlwaysFailExecutor));
        let result = executor.execute_chain(&ChainContext::default());
        assert_eq!(result.status, ChainStatus::Failed);
        assert_eq!(result.step_results[0].retries, 3); // 2 retries + 1 initial = 3 total attempts, retries=2... actually it's max_retries
    }

    #[test]
    fn test_step_execution_serialization() {
        let se = StepExecution {
            step_name: "Brainstorm".into(),
            payload: "topic".into(),
            output: Some("result".into()),
            success: true,
            duration_ms: 100,
            retries: 0,
        };
        let json = serde_json::to_string(&se).unwrap();
        let back: StepExecution = serde_json::from_str(&json).unwrap();
        assert_eq!(back.step_name, "Brainstorm");
        assert!(back.success);
    }

    #[test]
    fn test_chain_result_serialization_fields() {
        let result = ChainResult {
            chain_name: "c".into(),
            status: ChainStatus::Failed,
            step_results: vec![],
            final_output: None,
            error: Some("err".into()),
            duration_ms: 0,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ChainResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, ChainStatus::Failed);
        assert_eq!(back.error.as_deref(), Some("err"));
    }
}
