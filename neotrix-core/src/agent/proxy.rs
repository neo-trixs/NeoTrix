use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::CapabilityVector;
use crate::neotrix::nt_mind::SelfIteratingBrain;

/// 轻量级代理 — 隔离 CLI/server 对 SelfIteratingBrain 的直接依赖
/// 所有 neotrix 内部类型的引用限制在此模块内
pub struct ProxyBrain {
    inner: Arc<RwLock<SelfIteratingBrain>>,
}

#[derive(Debug, Clone)]
pub struct ProxyStats {
    pub capability_sum: f64,
    pub iteration: u64,
    pub absorb_count: u64,
    pub memory_count: usize,
    pub engine_active: bool,
    pub learning_rate: f64,
    pub capability: CapabilityVector,
}

impl ProxyBrain {
    pub fn new(inner: Arc<RwLock<SelfIteratingBrain>>) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &Arc<RwLock<SelfIteratingBrain>> {
        &self.inner
    }

    pub fn stats_blocking(&self) -> ProxyStats {
        let guard = self.inner.blocking_read();
        ProxyStats::from_sib(&guard)
    }

    pub async fn stats(&self) -> ProxyStats {
        let guard = self.inner.read().await;
        ProxyStats::from_sib(&guard)
    }

    pub async fn reason(&self, prompt: &str) -> Result<String, String> {
        use crate::neotrix::nt_world_model::TaskType;
        let mut agent = self.inner.write().await;
        if let Some(ref mut engine) = agent.reasoning_engine {
            engine.reason(prompt).map_err(|e| e.to_string())
        } else {
            let _result = agent.iterate(TaskType::General);
            Ok("(evolve)".to_string())
        }
    }

    pub async fn iterate(&self, task: &str) -> String {
        use crate::neotrix::nt_world_model::TaskType;
        let tt = if task == "design" { TaskType::Design } else { TaskType::General };
        let mut agent = self.inner.write().await;
        let before = agent.brain.capability.arr().iter().sum::<f64>();
        let result = agent.iterate(tt);
        let after = agent.brain.capability.arr().iter().sum::<f64>();
        format!("{:.3} → {:.3} (iter #{})", before, after, result.iteration)
    }

    pub async fn run_seal_loop(&self, desc: &str, reward: Option<f64>) -> Result<String, String> {
        let mut agent = self.inner.write().await;
        agent.run_seal_loop(desc, None, reward).map(|r| format!("{:.6}", r)).map_err(|e| e.to_string())
    }

    pub async fn absorb_sources(&self, sources: &[&str]) -> String {
        use crate::neotrix::nt_mind::KnowledgeSource;
        let mut agent = self.inner.write().await;
        let mut count = 0u32;
        for name in sources {
            let ks = match *name {
                "hero" | "heroid" => Some(KnowledgeSource::HeroUI),
                "base" | "baseui" => Some(KnowledgeSource::BaseUI),
                "arc" | "arcui" => Some(KnowledgeSource::ArcUI),
                "cortex" | "cortexui" => Some(KnowledgeSource::CortexUI),
                "agentic" | "agenticds" => Some(KnowledgeSource::AgenticDS),
                "hyper" | "hyperframes" => Some(KnowledgeSource::Hyperframes),
                "security" | "yao" => Some(KnowledgeSource::YaoWebsecurity),
                "bot" | "botasaurus" => Some(KnowledgeSource::Botasaurus),
                "react" | "reactdoctor" => Some(KnowledgeSource::ReactDoctor),
                "everos" | "hypergraph" => Some(KnowledgeSource::EverOS),
                "matt" | "mattpocock" => Some(KnowledgeSource::MattPocockSkills),
                "nested" | "nestedlearning" => Some(KnowledgeSource::NestedLearning),
                _ => None,
            };
            if let Some(ks) = ks {
                agent.brain.absorb(ks);
                count += 1;
            }
        }
        let _ = agent.brain.save();
        format!("absorbed {} sources", count)
    }

    pub async fn save(&self) -> Result<(), String> {
        let agent = self.inner.read().await;
        agent.brain.save().map_err(|e| e.to_string())
    }

    pub fn clone_inner(&self) -> Arc<RwLock<SelfIteratingBrain>> {
        self.inner.clone()
    }
}

impl ProxyStats {
    fn from_sib(sib: &SelfIteratingBrain) -> Self {
        let capability_sum: f64 = sib.brain.capability.arr().iter().sum();
        let absorb_count = sib.brain.total_absorb_count;
        let learning_rate = sib.brain.learning_rate;
        let capability = sib.brain.capability.clone();
        let memory_count = sib.reasoning_bank.memories().len();
        ProxyStats {
            capability_sum,
            iteration: sib.iteration,
            absorb_count,
            memory_count,
            engine_active: sib.reasoning_engine.is_some(),
            learning_rate,
            capability,
        }
    }
}

impl Clone for ProxyBrain {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}
