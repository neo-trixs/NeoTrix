//! 技能结晶引擎 — 从模板到正式技能
//!
//! 当技能模板成功 3+ 次时，自动生成可复用的子代理模板，
//! 并附带清晰的 I/O 契约（输入 schema、输出 schema、前置技能）。

use std::collections::HashMap;

/// 结晶状态
#[allow(dead_code)]
pub enum CrystallizationStatus {
    /// 模板阶段（成功 < 3 次）
    Template { success_count: u32 },
    /// 候选阶段（成功 3+ 次，待审批）
    Candidate { confidence: f64 },
    /// 已结晶（成为正式技能）
    Crystallized { skill_id: String },
    /// 已拒绝（质量不达标）
    Rejected { reason: String },
}

/// I/O 契约 — 定义技能的输入输出规范
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct IoContract {
    /// 输入 JSON schema
    pub input_schema: String,
    /// 输出 JSON schema
    pub output_schema: String,
    /// 前置技能列表
    pub prerequisites: Vec<String>,
    /// 预估 token 消耗
    pub estimated_tokens: u32,
    /// 预估延迟（毫秒）
    pub estimated_latency_ms: u32,
}

/// 已结晶技能 — 从模板晋升为正式技能
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CrystallizedSkill {
    /// 技能唯一标识
    pub id: String,
    /// 技能名称
    pub name: String,
    /// 技能描述
    pub description: String,
    /// I/O 契约
    pub contract: IoContract,
    /// 成功率
    pub success_rate: f64,
    /// 总调用次数
    pub total_invocations: u32,
    /// 结晶时间戳（Unix seconds）
    pub crystallized_at: i64,
    /// 来源模板 ID 列表
    pub source_templates: Vec<String>,
}

/// 结晶统计信息
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CrystallizationStats {
    /// 候选模板数量
    pub candidates: usize,
    /// 已结晶技能数量
    pub crystallized: usize,
    /// 已拒绝模板数量
    pub rejected: usize,
    /// 平均成功率
    pub avg_success_rate: f64,
}

/// 技能结晶引擎 — 监控模板成功率，自动结晶为正式技能
#[allow(dead_code)]
pub struct CrystallizationEngine {
    /// 待结晶模板 (template_id → success_count)
    candidates: HashMap<String, u32>,
    /// 已结晶技能
    crystallized: Vec<CrystallizedSkill>,
    /// 被拒绝的模板 (template_id, reason)
    rejected: Vec<(String, String)>,
    /// 结晶阈值（默认 3）
    threshold: u32,
    /// 下一个技能 ID 计数器
    next_skill_id: u32,
}

impl CrystallizationEngine {
    /// 创建新的结晶引擎，指定结晶阈值
    pub fn new(threshold: u32) -> Self {
        Self {
            candidates: HashMap::new(),
            crystallized: Vec::new(),
            rejected: Vec::new(),
            threshold,
            next_skill_id: 0,
        }
    }

    /// 记录模板成功一次
    pub fn record_success(&mut self, template_id: &str) {
        let count = self.candidates.entry(template_id.to_string()).or_insert(0);
        *count += 1;
    }

    /// 获取达到阈值的候选列表
    pub fn candidates(&self) -> Vec<(&str, u32)> {
        self.candidates
            .iter()
            .filter(|(_, count)| **count >= self.threshold)
            .map(|(id, count)| (id.as_str(), *count))
            .collect()
    }

    /// 获取模板当前状态
    pub fn status_of(&self, template_id: &str) -> CrystallizationStatus {
        match self.candidates.get(template_id) {
            Some(&count) if count >= self.threshold => {
                let confidence = (count as f64 / self.threshold as f64).min(1.0);
                CrystallizationStatus::Candidate { confidence }
            }
            Some(&count) => CrystallizationStatus::Template {
                success_count: count,
            },
            None => {
                if self.crystallized.iter().any(|s| s.source_templates.contains(&template_id.to_string())) {
                    let skill_id = self
                        .crystallized
                        .iter()
                        .find(|s| s.source_templates.contains(&template_id.to_string()))
                        .unwrap()
                        .id
                        .clone();
                    CrystallizationStatus::Crystallized { skill_id }
                } else if self.rejected.iter().any(|(id, _)| id == template_id) {
                    let reason = self
                        .rejected
                        .iter()
                        .find(|(id, _)| id == template_id)
                        .unwrap()
                        .1
                        .clone();
                    CrystallizationStatus::Rejected { reason }
                } else {
                    CrystallizationStatus::Template {
                        success_count: 0,
                    }
                }
            }
        }
    }

    /// 结晶一个候选为正式技能
    pub fn crystallize(
        &mut self,
        template_id: &str,
        name: &str,
        description: &str,
        contract: IoContract,
    ) -> Option<CrystallizedSkill> {
        let success_count = match self.candidates.get(template_id) {
            Some(&c) if c >= self.threshold => c,
            _ => return None,
        };

        self.next_skill_id += 1;
        let skill = CrystallizedSkill {
            id: format!("skill_{}", self.next_skill_id),
            name: name.to_string(),
            description: description.to_string(),
            contract,
            success_rate: 1.0,
            total_invocations: success_count,
            crystallized_at: chrono_now(),
            source_templates: vec![template_id.to_string()],
        };

        self.crystallized.push(skill.clone());
        self.candidates.remove(template_id);
        Some(skill)
    }

    /// 拒绝一个候选
    pub fn reject(&mut self, template_id: &str, reason: &str) {
        self.rejected
            .push((template_id.to_string(), reason.to_string()));
        self.candidates.remove(template_id);
    }

    /// 获取所有已结晶技能
    pub fn crystallized_skills(&self) -> &[CrystallizedSkill] {
        &self.crystallized
    }

    /// 获取统计信息
    pub fn stats(&self) -> CrystallizationStats {
        CrystallizationStats {
            candidates: self.candidates.len(),
            crystallized: self.crystallized.len(),
            rejected: self.rejected.len(),
            avg_success_rate: if self.crystallized.is_empty() {
                0.0
            } else {
                self.crystallized.iter().map(|s| s.success_rate).sum::<f64>()
                    / self.crystallized.len() as f64
            },
        }
    }

    /// 生成 SKILL.md 格式模板
    pub fn to_skill_md(&self, skill: &CrystallizedSkill) -> String {
        let prereqs = if skill.contract.prerequisites.is_empty() {
            "None".to_string()
        } else {
            skill.contract.prerequisites.join(", ")
        };

        format!(
            "# {}\n\n## Description\n{}\n\n## Input\n```json\n{}\n```\n\n## Output\n```json\n{}\n```\n\n## Prerequisites\n{}\n\n## Stats\n- Success Rate: {:.1}%\n- Invocations: {}\n- Crystallized: {}\n",
            skill.name,
            skill.description,
            skill.contract.input_schema,
            skill.contract.output_schema,
            prereqs,
            skill.success_rate * 100.0,
            skill.total_invocations,
            skill.crystallized_at,
        )
    }

    /// 合并另一个结晶引擎的结果（用于跨会话持久化后恢复）
    pub fn merge(&mut self, other: &CrystallizationEngine) {
        for (id, count) in &other.candidates {
            let entry = self.candidates.entry(id.clone()).or_insert(0);
            *entry = (*entry).max(*count);
        }
        for skill in &other.crystallized {
            if !self.crystallized.iter().any(|s| s.id == skill.id) {
                self.crystallized.push(skill.clone());
            }
        }
        for rejected in &other.rejected {
            if !self.rejected.iter().any(|(id, _)| id == &rejected.0) {
                self.rejected.push(rejected.clone());
            }
        }
    }
}

/// 获取当前 Unix 时间戳（秒）
fn chrono_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_contract() -> IoContract {
        IoContract {
            input_schema: r#"{"type":"object","properties":{"text":{"type":"string"}}"#.to_string(),
            output_schema: r#"{"type":"object","properties":{"result":{"type":"string"}}"#.to_string(),
            prerequisites: vec!["base_nlp".to_string()],
            estimated_tokens: 500,
            estimated_latency_ms: 200,
        }
    }

    #[test]
    fn test_record_success_and_candidates() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        assert!(engine.candidates().is_empty());

        engine.record_success("tmpl_a");
        let cands = engine.candidates();
        assert_eq!(cands.len(), 1);
        assert_eq!(cands[0], ("tmpl_a", 3));
    }

    #[test]
    fn test_crystallize() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        let skill = engine
            .crystallize("tmpl_a", "Test Skill", "A test skill", test_contract())
            .unwrap();
        assert_eq!(skill.id, "skill_1");
        assert_eq!(skill.name, "Test Skill");
        assert_eq!(skill.total_invocations, 3);
        assert!(engine.candidates().is_empty());
        assert_eq!(engine.crystallized_skills().len(), 1);
    }

    #[test]
    fn test_crystallize_below_threshold() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        let result = engine.crystallize("tmpl_a", "Fail", "Should fail", test_contract());
        assert!(result.is_none());
    }

    #[test]
    fn test_reject() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");
        engine.record_success("tmpl_a");

        engine.reject("tmpl_a", "质量不达标");
        assert!(engine.candidates().is_empty());
        assert_eq!(engine.rejected.len(), 1);

        let status = engine.status_of("tmpl_a");
        match status {
            CrystallizationStatus::Rejected { reason } => assert_eq!(reason, "质量不达标"),
            _ => panic!("Expected Rejected"),
        }
    }

    #[test]
    fn test_stats() {
        let mut engine = CrystallizationEngine::new(3);
        engine.record_success("a");
        engine.record_success("a");
        engine.record_success("a");
        engine.crystallize("a", "A", "desc", test_contract());

        let stats = engine.stats();
        assert_eq!(stats.crystallized, 1);
        assert_eq!(stats.avg_success_rate, 1.0);
    }

    #[test]
    fn test_to_skill_md() {
        let engine = CrystallizationEngine::new(3);
        let skill = CrystallizedSkill {
            id: "skill_1".to_string(),
            name: "My Skill".to_string(),
            description: "Does things".to_string(),
            contract: test_contract(),
            success_rate: 0.95,
            total_invocations: 10,
            crystallized_at: 1700000000,
            source_templates: vec!["tmpl_1".to_string()],
        };
        let md = engine.to_skill_md(&skill);
        assert!(md.contains("# My Skill"));
        assert!(md.contains("95.0%"));
        assert!(md.contains("base_nlp"));
    }
}
