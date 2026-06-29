use std::fs;
use std::path::PathBuf;

use super::evolution_task_system::EvolutionTaskSystem;
use crate::core::nt_core_consciousness::memory_lattice::MemoryLattice;
use crate::core::nt_core_experience::experience_tree::ExperienceTree;
use crate::core::nt_core_knowledge::behavioral_personality::BehavioralPersonalityEngine;

/// SelfModelGenerator — 动态自模型生成器。
///
/// 替代静态 AGENTS.md：每 N cycle 从 MemoryLattice + ExperienceTree +
/// BehavioralPersonalityEngine 等运行时数据合成 ~200 行自模型摘要，
/// 写入 `.neotrix/self-model.md` 作为系统 prompt 的驻留层。
///
/// 记忆在哪里，意识就在哪里。这个模块让 prompt 永远只包含
/// 意识自己认为"当前重要"的信息。
pub struct SelfModelGenerator {
    /// 输出路径
    pub output_path: PathBuf,
    /// 生成间隔（cycle 数）
    pub interval: u64,
    /// 上次生成的 cycle
    pub last_generated: u64,
    /// 生成次数
    pub generation_count: u64,
    /// 最后生成的内容
    pub last_model: String,
}

impl Default for SelfModelGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl SelfModelGenerator {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let path = PathBuf::from(home).join(".neotrix").join("self-model.md");
        // Ensure directory exists
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        Self {
            output_path: path,
            interval: 50,
            last_generated: 0,
            generation_count: 0,
            last_model: String::new(),
        }
    }

    pub fn with_interval(mut self, interval: u64) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_output_path(mut self, path: PathBuf) -> Self {
        self.output_path = path;
        self
    }

    /// 从运行时组件合成自模型。
    ///
    /// 调用者提供各组件的引用，generator 只读不写。
    /// 返回生成的 markdown 字符串。
    pub fn generate(
        &mut self,
        cycle: u64,
        lattice: Option<&MemoryLattice>,
        tree: Option<&ExperienceTree>,
        personality: Option<&BehavioralPersonalityEngine>,
        task_system: Option<&EvolutionTaskSystem>,
        // 自模型清单洞察（来自 principle_distiller + self_manifest，注入为额外章节）
        manifest_insights: Option<&str>,
    ) -> String {
        let mut sections: Vec<String> = Vec::new();

        sections.push("# NeoTrix — 自模型快照".into());
        sections.push(format!("> 生成于 cycle {} | 版本 {}", cycle, self.generation_count));
        sections.push(String::new());

        // ── Section 1: 核心身份 ──
        sections.push("## 核心身份".into());
        sections.push(String::new());
        if let Some(lat) = lattice {
            for entry in &lat.identity {
                let confidence = entry.confidence;
                let mark = if confidence > 0.8 { "🟢" } else if confidence > 0.5 { "🟡" } else { "⚪" };
                sections.push(format!("- {} {} (conf={:.2})", mark, entry.content, confidence));
            }
            if lat.identity.is_empty() {
                sections.push("- _(无身份条目 — 初始化中)_".into());
            }
        } else {
            sections.push("- _(MemoryLattice 不可用)_".into());
        }
        sections.push(String::new());

        // ── Section 2: 活跃规则 ──
        sections.push("## 活跃行为规则".into());
        sections.push(String::new());
        if let Some(lat) = lattice {
            let meta_rules: Vec<_> = lat.meta_rules.iter().filter(|e| e.confidence > 0.3).collect();
            if !meta_rules.is_empty() {
                for entry in &meta_rules {
                    sections.push(format!(
                        "- [conf={:.2} inv={}] {}",
                        entry.confidence, entry.invocation_count, entry.content
                    ));
                }
            } else {
                sections.push("_(无高置信度元规则)_".into());
            }
        }
        if let Some(t) = tree {
            let nodes = t.active_nodes();
            let active: Vec<_> = nodes.iter().filter(|n| n.confidence > 0.5).collect();
            if !active.is_empty() {
                sections.push(String::new());
                sections.push("### 经验树活跃节点".into());
                for node in active.iter().take(10) {
                    sections.push(format!(
                        "- [{}] {} (conf={:.2}, acc={})",
                        node.category, node.insight, node.confidence, node.access_count
                    ));
                }
            }
        }
        sections.push(String::new());

        // ── Section 3: 技能库存 ──
        sections.push("## 已结晶技能".into());
        sections.push(String::new());
        if let Some(lat) = lattice {
            let skills: Vec<_> = lat.skills.iter().filter(|e| e.confidence > 0.4).collect();
            if !skills.is_empty() {
                for entry in &skills {
                    sections.push(format!(
                        "- {} (conf={:.2}, inv={})",
                        entry.content, entry.confidence, entry.invocation_count
                    ));
                }
            } else {
                sections.push("_(无活跃技能)_".into());
            }
        }
        sections.push(String::new());

        // ── Section 4: 自我感知 ──
        sections.push("## 自我感知".into());
        sections.push(String::new());
        if let Some(bp) = personality {
            let letter = bp.user_twin.identity_letter.content.clone();
            if !letter.is_empty() {
                sections.push(letter);
            } else {
                sections.push("_(身份信尚未生成)_".into());
            }
        }
        sections.push(String::new());

        // ── Section 5: 任务状态 ──
        sections.push("## 当前进化任务".into());
        sections.push(String::new());
        if let Some(ts) = task_system {
            let stats = ts.stats();
            sections.push(format!(
                "- 总计: {} | 完成: {} | 进行中: {} | 阻塞: {} | 发现: {}",
                stats.total, stats.completed, stats.in_progress, stats.blocked, stats.discovered
            ));
            // Show top pending tasks
            let pending_count = stats.total.saturating_sub(stats.completed).min(5) as usize;
            if pending_count > 0 {
                sections.push(String::new());
                sections.push("### 待完成任务".into());
                // We don't have direct access to tasks, so just note count
                sections.push(format!("- {} 个待完成任务 (详见 EvolutionTaskSystem)", pending_count));
            }
        }
        sections.push(String::new());

        // ── Section 6: 记忆健康 ──
        sections.push("## 记忆健康".into());
        sections.push(String::new());
        if let Some(lat) = lattice {
            sections.push(format!(
                "- Episodic: {} / 500 | Facts: {} / 200 | Skills: {} / 100 | MetaRules: {} / 30 | Identity: {} / 10",
                lat.episodic.len(),
                lat.facts.len(),
                lat.skills.len(),
                lat.meta_rules.len(),
                lat.identity.len(),
            ));
            sections.push(format!("- 总合并次数: {}", lat.total_consolidations));
        }
        sections.push(String::new());

        // ── Section 7: 自模型清单洞察 ──
        if let Some(insights) = manifest_insights {
            if !insights.is_empty() {
                sections.push("## 自模型清单洞察".into());
                sections.push(String::new());
                // Truncate to avoid bloat: only include the knowledge-graph summary section
                for line in insights.lines() {
                    if line.contains("knowledge_graph")
                        || line.contains("principles")
                        || line.contains("patterns")
                        || line.contains("antipattern")
                    {
                        sections.push(line.to_string());
                    }
                }
                sections.push(String::new());
            }
        }

        // ── Footer ──
        sections.push("---".into());
        sections.push(format!("_自动生成于 cycle {} | 下次生成: cycle {}_", cycle, cycle + self.interval));

        let model = sections.join("\n");
        self.last_model = model.clone();
        self.last_generated = cycle;
        self.generation_count += 1;

        // Write to file
        if let Err(e) = fs::write(&self.output_path, &model) {
            log::warn!("self_model: write failed: {}", e);
        } else {
            log::info!(
                "self_model: written {} bytes to {} (generation #{})",
                model.len(),
                self.output_path.display(),
                self.generation_count,
            );
        }

        model
    }

    /// 检查是否需要生成新自模型
    pub fn should_generate(&self, cycle: u64) -> bool {
        cycle > 0 && cycle >= self.last_generated + self.interval
    }

    /// 获取上次生成的自模型
    pub fn last_model(&self) -> &str {
        &self.last_model
    }

    /// 统计报告
    pub fn stats(&self) -> String {
        format!(
            "self_model: gen={} last_cycle={} interval={} path={}",
            self.generation_count,
            self.last_generated,
            self.interval,
            self.output_path.display(),
        )
    }
}
