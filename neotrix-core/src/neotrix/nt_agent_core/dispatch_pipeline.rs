use super::permission::{PermissionDecision, PermissionGate};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineResult {
    Allowed,
    Denied(String),
    RetryNeeded,
    Escalated,
}

impl PipelineResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, PipelineResult::Allowed)
    }
}

pub struct DispatchContext<'a> {
    pub handler: &'a str,
    pub permission_gate: &'a PermissionGate,
}

pub trait PipelineStage {
    fn name(&self) -> &'static str;
    fn execute(&self, ctx: &DispatchContext) -> PipelineResult;
}

pub struct ApprovalStage;

impl PipelineStage for ApprovalStage {
    fn name(&self) -> &'static str {
        "approval"
    }

    fn execute(&self, ctx: &DispatchContext) -> PipelineResult {
        match ctx.permission_gate.check(
            ctx.handler,
            &super::message::AgentId::new("pipeline", "1.0"),
        ) {
            PermissionDecision::Allow => PipelineResult::Allowed,
            PermissionDecision::Deny(r) => PipelineResult::Denied(r),
            PermissionDecision::AskHuman => {
                PipelineResult::Denied("requires human approval".into())
            }
        }
    }
}

pub struct ToolDispatchPipeline {
    stages: Vec<Box<dyn PipelineStage>>,
    max_retries: u32,
}

impl ToolDispatchPipeline {
    pub fn new() -> Self {
        Self {
            stages: vec![Box::new(ApprovalStage)],
            max_retries: 2,
        }
    }

    pub fn with_stage(mut self, stage: Box<dyn PipelineStage>) -> Self {
        self.stages.push(stage);
        self
    }

    pub fn execute(&self, ctx: &DispatchContext) -> PipelineResult {
        for stage in &self.stages {
            match stage.execute(ctx) {
                PipelineResult::Allowed => continue,
                denied @ (PipelineResult::Denied(_) | PipelineResult::Escalated) => return denied,
                PipelineResult::RetryNeeded => {
                    for attempt in 0..self.max_retries {
                        match stage.execute(ctx) {
                            PipelineResult::Allowed => break,
                            _ if attempt < self.max_retries - 1 => continue,
                            result => return result,
                        }
                    }
                }
            }
        }
        PipelineResult::Allowed
    }

    pub fn with_max_retries(mut self, n: u32) -> Self {
        self.max_retries = n;
        self
    }
}

impl Default for ToolDispatchPipeline {
    fn default() -> Self {
        Self::new()
    }
}
