//! 多 ReasoningBrain 协同模块
//! 借鉴 MemOS 多记忆体思想，支持多个 ReasoningBrain 实例协同工作
//!
//! 设计思想：
//! - 每个 ReasoningBrain 可以专注于特定任务类型
//! - 通过 ReasoningBank 共享知识（MicroEdit 序列）
//! - 支持 brain 之间的知识迁移和融合
//! - 借鉴 dbskill：多维度相似度计算，选择最匹配的 brain

use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use super::core::KnowledgeSource;
use super::self_edit::MicroEdit;

/// 多 Brain 管理器
/// 借鉴 MemOS：管理多个记忆体（这里对应多个 ReasoningBrain）
pub struct MultiBrainManager {
    /// 活跃的 Brain 实例
    brains: Vec<ReasoningBrain>,
    
    /// 共享的 ReasoningBank（所有 brain 共享知识库）
    shared_bank: ReasoningBank,
    
    /// Brain 到任务类型的映射
    brain_task_map: Vec<(usize, String)>, // (brain_index, task_type)
}

impl MultiBrainManager {
    /// 创建新的多 Brain 管理器
    pub fn new(bank_capacity: usize) -> Self {
        Self {
            brains: Vec::new(),
            shared_bank: ReasoningBank::new(bank_capacity),
            brain_task_map: Vec::new(),
        }
    }
    
    /// 添加一个新的 ReasoningBrain
    /// 借鉴 MemOS：为每个记忆体分配领域
    pub fn add_brain(&mut self, brain: ReasoningBrain, task_type: &str) {
        let index = self.brains.len();
        self.brains.push(brain);
        self.brain_task_map.push((index, task_type.to_string()));
    }
    
    /// 根据任务类型选择合适的 Brain
    /// 借鉴 dbskill：多维度检索，选择最匹配的 brain
    pub fn select_brain(&self, task_type: &str) -> Option<&ReasoningBrain> {
        self.brain_task_map
            .iter()
            .find(|(_, t)| t == task_type)
            .map(|(idx, _)| &self.brains[*idx])
    }
    
    /// 选择最匹配的 Brain（基于能力向量相似度）
    /// 借鉴 dbskill：多维度检索，使用 cosine similarity
    pub fn select_brain_by_capability(&self, target: &ReasoningBrain) -> Option<(usize, f64)> {
        self.brains.iter()
            .enumerate()
            .map(|(i, brain)| {
                let sim = brain.capability.similarity(&target.capability);
                (i, sim)
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, sim)| *sim > 0.5)
    }
    
    /// 协同更新：将一个 brain 的知识迁移到另一个
    /// 借鉴 MemOS 记忆迁移：提取 source brain 的 KnowledgeSource 并应用到 target brain
    pub fn migrate_knowledge(&mut self, from_index: usize, to_index: usize) {
        if from_index >= self.brains.len() || to_index >= self.brains.len() {
            return;
        }
        
        // 从 source brain 提取知识来源
        let sources: Vec<KnowledgeSource> = self.brains[from_index].absorption_history
            .iter()
            .map(|r| r.source)
            .collect();
        
        if sources.is_empty() {
            return;
        }
        
        // 应用到 target brain
        self.brains[to_index].absorb_batch(&sources);
    }
    
    /// 合并所有 brain 的知识到共享 ReasoningBank
    /// 每个 brain 的 CapabilityVector 转化为 ReasoningMemory 存储
    pub fn consolidate_knowledge(&mut self) {
        for (i, brain) in self.brains.iter().enumerate() {
            // 将 CapabilityVector 转化为 MicroEdit 序列
            let mut micro_edits = Vec::new();
            let task_type_name = self.brain_task_map.get(i)
                .map(|(_, t)| t.clone())
                .unwrap_or_else(|| "unknown".to_string());
            
            // 从 absorption_history 提取知识来源作为 micro_edits 的调整量
            for record in &brain.absorption_history {
                let source_name = match record.source {
                    KnowledgeSource::HeroUI => "compound_composition",
                    KnowledgeSource::BaseUI => "accessibility",
                    KnowledgeSource::ArcUI => "ai_native_states",
                    KnowledgeSource::CortexUI => "semantic_layer",
                    KnowledgeSource::AgenticDS => "quality_gates",
                    KnowledgeSource::DesignPhilosophy => "typography",
                    _ => "other",
                };
                micro_edits.push(MicroEdit::AdjustDimension(
                    source_name.to_string(),
                    record.weight * brain.capability.arr().iter().sum::<f64>() / 23.0,
                ));
            }
            
            micro_edits.push(MicroEdit::NormalizeVector);
            
            let memory = ReasoningMemory::new(
                &format!("consolidated from brain {} ({})", i, task_type_name),
                crate::neotrix::nt_world_model::TaskType::General,
                &micro_edits,
                brain.capability.arr().iter().sum::<f64>() / 23.0,
            );
            
            self.shared_bank.store(memory);
        }
    }
    
    /// 获取共享 ReasoningBank 的引用
    pub fn shared_bank(&self) -> &ReasoningBank {
        &self.shared_bank
    }
    
    /// 获取共享 ReasoningBank 的可变引用
    pub fn shared_bank_mut(&mut self) -> &mut ReasoningBank {
        &mut self.shared_bank
    }
    
    /// 获取所有 brain 的引用
    pub fn brains(&self) -> &Vec<ReasoningBrain> {
        &self.brains
    }
    
    /// 获取 brain 数量
    pub fn brain_count(&self) -> usize {
        self.brains.len()
    }
    
    /// 获取某个 brain 的可变引用
    pub fn brain_mut(&mut self, index: usize) -> Option<&mut ReasoningBrain> {
        self.brains.get_mut(index)
    }

    /// GEA 风格群体进化：找出最优 brain，将其知识广播到其他 brain
    /// 最优 = total_absorb_count 最高的 brain
    /// 这是群体级知识共享的简化实现
    pub fn evolve_group(&mut self) {
        if self.brains.len() < 2 { return; }

        // 选择最优 brain
        let best_idx = self.brains.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_absorb_count.cmp(&b.total_absorb_count))
            .map(|(i, _)| i);

        if let Some(idx) = best_idx {
            let best_absorbed = self.brains[idx].total_absorb_count;
            for i in 0..self.brains.len() {
                if i != idx && self.brains[i].total_absorb_count < best_absorbed {
                    self.migrate_knowledge(idx, i);
                }
            }
        }

        // 合并所有 brain 知识到共享 bank
        self.consolidate_knowledge();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_multi_brain_creation() {
        let manager = MultiBrainManager::new(100);
        assert_eq!(manager.brain_count(), 0);
    }
    
    #[test]
    fn test_add_and_select_brain() {
        let mut manager = MultiBrainManager::new(100);
        let brain1 = ReasoningBrain::new();
        let brain2 = ReasoningBrain::new();
        
        manager.add_brain(brain1, "UIDesign");
        manager.add_brain(brain2, "CodeAnalysis");
        
        assert_eq!(manager.brain_count(), 2);
        
        let selected = manager.select_brain("UIDesign");
        assert!(selected.is_some());
        
        let not_found = manager.select_brain("NonExistent");
        assert!(not_found.is_none());
    }
    
    #[test]
    fn test_migrate_knowledge() {
        let mut manager = MultiBrainManager::new(100);
        let mut brain1 = ReasoningBrain::new();
        let brain2 = ReasoningBrain::new();
        
        // 给 brain1 吸收一些知识
        brain1.absorb(KnowledgeSource::HeroUI);
        brain1.absorb(KnowledgeSource::BaseUI);
        
        manager.add_brain(brain1, "UIDesign");
        manager.add_brain(brain2, "CodeAnalysis");
        
        // 迁移知识从 brain 0 到 brain 1
        manager.migrate_knowledge(0, 1);
        
        let target_brain = manager.brain_mut(1).expect("value should be ok in test");
        assert!(target_brain.total_absorb_count >= 2);
    }
    
    #[test]
    fn test_consolidate_knowledge() {
        let mut manager = MultiBrainManager::new(100);
        let mut brain1 = ReasoningBrain::new();
        let mut brain2 = ReasoningBrain::new();
        
        brain1.absorb(KnowledgeSource::HeroUI);
        brain2.absorb(KnowledgeSource::BaseUI);
        
        manager.add_brain(brain1, "UIDesign");
        manager.add_brain(brain2, "Accessibility");
        
        // 初始共享 bank 为空
        assert_eq!(manager.shared_bank().stats().total_memories, 0);
        
        // 合并知识
        manager.consolidate_knowledge();
        
        // 应该有 2 个记忆（每个 brain 一个）
        assert_eq!(manager.shared_bank().stats().total_memories, 2);
    }
}
