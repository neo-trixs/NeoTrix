//! Pipeline Scheduler — 渐进式记忆提取管线 (L1→L2→L3)
//!
//! 受 TencentDB Agent Memory 启发: 原始记忆(L0) → 原子记忆(L1) → 场景归纳(L2) → 用户画像(L3)
//! Context Offloading: 冗长轨迹卸载到 refs/*.md, 上下文仅保留 Mermaid 任务图
//!
//! 时序驱动: L1 每 N 个新记忆, L2 每 M 次 L1, L3 每 P 次 L2

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;

use crate::core::nt_core_bank::{
    L1Memory, OffloadManager, Persona, PipelineConfig, PipelineState, ReasoningBank,
    ReasoningMemory, SceneBlock,
};

/// 流水线调度器
pub struct PipelineScheduler {
    pub config: PipelineConfig,
    pub state: RwLock<PipelineState>,
    offload: RwLock<OffloadManager>,
    running: std::sync::atomic::AtomicBool,
}

impl PipelineScheduler {
    pub fn new(base_path: &std::path::Path) -> Self {
        Self {
            config: PipelineConfig::default(),
            state: RwLock::new(PipelineState::new()),
            offload: RwLock::new(OffloadManager::new(base_path)),
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// 记录新记忆并检查是否触发管线
    pub async fn record_and_tick(&self, mem: &ReasoningMemory, bank: &mut ReasoningBank) {
        // 卸载到 refs/*.md
        {
            let mut offload = self.offload.write().await;
            let _ = offload.offload_memory(mem);
        }

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.record_memory();
        }

        // 检查并触发 L1
        if self.state.read().await.should_trigger_l1(&self.config) {
            self.trigger_l1(bank).await;
        }
    }

    /// L1: 提取原子记忆
    pub async fn trigger_l1(&self, bank: &mut ReasoningBank) {
        let memories: Vec<ReasoningMemory> = bank.memories().iter().cloned().collect();
        if memories.is_empty() {
            return;
        }

        // 使用 T3 视图作为 L1 提取的输入线索
        let l1_results: Vec<L1Memory> = memories
            .iter()
            .filter_map(|m| {
                let content = m
                    .t3_views
                    .semantic_view
                    .as_ref()
                    .or(m.t3_views.struct_view.as_ref())?;
                let mem_type = if m.task_description.contains("nt_shield")
                    || m.task_description.contains("safety")
                {
                    "instruction"
                } else if m.reward > 0.7 {
                    "episodic"
                } else {
                    "persona"
                };
                Some(L1Memory {
                    id: uuid::Uuid::new_v4().to_string(),
                    content: content.clone(),
                    mem_type: mem_type.to_string(),
                    priority: m.reward,
                    source_memory_id: m.id.clone(),
                    created_at: chrono::Utc::now().timestamp(),
                })
            })
            .collect();

        if l1_results.is_empty() {
            return;
        }

        // 保存 L1
        {
            let offload = self.offload.write().await;
            let _ = offload.save_l1_extraction(&l1_results);
        }

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.l1_count += 1;
            state.pending_memories = 0;
            state.last_l1_time = chrono::Utc::now().timestamp();
        }

        // 检查 L2
        if self.state.read().await.should_trigger_l2(&self.config) {
            self.trigger_l2().await;
        }
    }

    /// L2: 场景归纳
    pub async fn trigger_l2(&self) {
        // 从 L1 文件读取已有原子记忆
        let l1_memories = self.load_l1_memories().await;
        if l1_memories.is_empty() {
            return;
        }

        // 按 mem_type + 时间聚类
        let mut scenes: Vec<SceneBlock> = Vec::new();
        for mem in &l1_memories {
            let existing = scenes.iter_mut().find(|s: &&mut SceneBlock| {
                s.memory_ids.len() < 10
                    && mem.content.contains(&s.summary[..s.summary.len().min(20)])
            });
            if let Some(scene) = existing {
                scene.memory_ids.push(mem.id.clone());
                scene.content.push(mem.content.clone());
                scene.heat += 1;
                scene.updated_at = chrono::Utc::now().timestamp();
            } else {
                scenes.push(SceneBlock {
                    id: format!("scene-{}", uuid::Uuid::new_v4()),
                    summary: if mem.content.len() > 60 {
                        mem.content[..60].to_string()
                    } else {
                        mem.content.clone()
                    },
                    content: vec![mem.content.clone()],
                    heat: 1,
                    created_at: chrono::Utc::now().timestamp(),
                    updated_at: chrono::Utc::now().timestamp(),
                    memory_ids: vec![mem.id.clone()],
                });
            }
        }

        // 保存场景
        {
            let offload = self.offload.write().await;
            for scene in &scenes {
                let _ = offload.save_scene(scene);
            }
        }

        // 更新状态
        {
            let mut state = self.state.write().await;
            state.l2_count += 1;
            state.last_l2_time = chrono::Utc::now().timestamp();
        }

        // 检查 L3
        if self.state.read().await.should_trigger_l3(&self.config) {
            self.trigger_l3().await;
        }
    }

    /// L3: 画像更新
    pub async fn trigger_l3(&self) {
        let existing = {
            let offload = self.offload.read().await;
            offload.load_persona()
        };

        // 从 T3 Views 提取画像特征
        let mut persona = Persona::new();
        if let Some(ref existing_str) = existing {
            for line in existing_str.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("## Base Anchors") {
                    persona.base_anchors.push(trimmed.to_string());
                }
                if trimmed.starts_with("## Interest Map") {
                    persona.interest_map.push(trimmed.to_string());
                }
                if trimmed.starts_with("## Interaction Protocol") {
                    persona.interaction_protocol.push(trimmed.to_string());
                }
                if trimmed.starts_with("## Cognitive Kernel") {
                    persona.cognitive_kernel.push(trimmed.to_string());
                }
            }
        }

        // 更新追加
        persona.base_anchors.push(format!(
            "Last pipeline run: {}",
            chrono::Utc::now().timestamp()
        ));
        persona
            .interest_map
            .push("Task types observed: General, Design, CodeAnalysis, Security".to_string());
        persona
            .interaction_protocol
            .push("Prefers structured output with reward signals".to_string());
        persona
            .cognitive_kernel
            .push("Adaptive learning via capability vector updates".to_string());

        {
            let offload = self.offload.write().await;
            let _ = offload.save_persona(&persona);
        }

        let mut state = self.state.write().await;
        state.l3_count += 1;
        state.last_l3_time = chrono::Utc::now().timestamp();
    }

    /// 从文件加载 L1 原子记忆
    async fn load_l1_memories(&self) -> Vec<L1Memory> {
        let path = self.offload.read().await.base_path.join("l1_atomic.jsonl");
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        content
            .lines()
            .filter_map(|line| {
                serde_json::from_str::<L1Memory>(line)
                    .inspect_err(|e| log::warn!("[pipeline] skip line: {}", e))
                    .ok()
            })
            .collect()
    }

    /// 加载当前画像 (供 ReasoningEngine 注入)
    pub async fn load_persona(&self) -> Option<String> {
        self.offload.read().await.load_persona()
    }

    /// 启动后台定时检查循环
    pub async fn start(self: Arc<Self>, bank: Arc<RwLock<ReasoningBank>>) {
        if self
            .running
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        loop {
            if !self.running.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            sleep(Duration::from_secs(60)).await;
            let should_l1 = self.state.read().await.should_trigger_l1(&self.config);
            if should_l1 {
                let mut bank = bank.write().await;
                self.trigger_l1(&mut bank).await;
            }
            // 空闲超时触发 (180s 无活动)
            let now = chrono::Utc::now().timestamp();
            let state = self.state.read().await;
            let idle = now - state.last_l1_time;
            if idle > 180 && state.pending_memories > 0 {
                drop(state);
                let mut bank = bank.write().await;
                self.trigger_l1(&mut bank).await;
            }
        }
    }

    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }
}
