use super::self_edit::MicroEdit;
use super::self_iterating::ReasoningBrain;
use super::memory::{ReasoningBank, ReasoningMemory};
use crate::core::nt_core_self::skill_crystal::{
    SkillCrystal, CrystalRegistry, VerificationContract,
};
use crate::core::nt_core_self::reasoning_strategy::StrategyKind;
use crate::core::nt_core_self::attention_head::AttentionDomain;
use crate::neotrix::nt_world_model::TaskType;

pub struct AutoCrystallizer {
    pub registry: CrystalRegistry,
    pub auto_crystallize: bool,
    pub min_reward_threshold: f64,
    pub total_crystallized: u64,
    /// 反幻觉门统计 (P0-1 吸收自 DeepZero assessment.j2):
    /// 无验证契约的结晶计数 — 认识论声明"没验证 ≠ 有效"。
    pub unverified_crystallized: u64,
    /// 反幻觉门开关：true 时无验证契约的结晶标记为 Unverified 并计数。
    pub anti_hallucination_gate: bool,
    /// P0-4 Hallucination Bin (吸收 bullyingllms 模式):
    /// 被门禁拒绝的结晶进幻觉桶 — 可审计、可追溯、可复查,
    /// 而非仅计数丢弃。bullyingllms 原文: "findings go into a
    /// hallucination bin before passing the gate"。
    pub hallucination_bin: Vec<HallucinationEntry>,
}

/// 幻觉桶条目 — 记录被反幻觉门拒绝的结晶候选及其拒绝原因。
#[derive(Debug, Clone)]
pub struct HallucinationEntry {
    pub source_name: String,
    pub domain: String,
    pub reward: f64,
    pub reason: String,
    pub timestamp: u64,
}

impl AutoCrystallizer {
    pub fn new() -> Self {
        Self {
            registry: CrystalRegistry::new(),
            auto_crystallize: true,
            min_reward_threshold: 0.3,
            total_crystallized: 0,
            unverified_crystallized: 0,
            anti_hallucination_gate: true,
            hallucination_bin: Vec::new(),
        }
    }

    /// 反幻觉门：为结晶附加验证契约。
    /// 无契约 → 标记 Unverified 并计数（认识论声明，不阻断结晶本身）。
    fn apply_verification_gate(
        &mut self,
        verification: Option<VerificationContract>,
    ) -> Option<VerificationContract> {
        match verification {
            Some(v) => Some(v),
            None => {
                if self.anti_hallucination_gate {
                    self.unverified_crystallized += 1;
                }
                None
            }
        }
    }

    /// P0-4 幻觉桶入桶 (bullyingllms 模式): 无验证契约的结晶候选
    /// 记录到 hallucination_bin, 供审计/复查 — 拒绝可追溯而非静默丢弃。
    pub fn bin_hallucination(&mut self, source_name: &str, domain: &str, reward: f64, reason: &str) {
        if !self.anti_hallucination_gate {
            return;
        }
        let now = crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_diversity::now_unix_secs() as u64;
        self.hallucination_bin.push(HallucinationEntry {
            source_name: source_name.to_string(),
            domain: domain.to_string(),
            reward,
            reason: reason.to_string(),
            timestamp: now,
        });
    }

    /// 幻觉桶审计: 返回当前桶内条目数 (供监控/复盘)。
    pub fn hallucination_bin_len(&self) -> usize {
        self.hallucination_bin.len()
    }

    pub fn crystallize_from_absorption(
        &mut self,
        _brain: &mut ReasoningBrain,
        bank: &mut ReasoningBank,
        source_url: &str,
        source_name: &str,
        domain: &str,
        edits: &[MicroEdit],
        reward: f64,
        verification: Option<VerificationContract>,
    ) -> Option<SkillCrystal> {
        if !self.auto_crystallize || reward < self.min_reward_threshold {
            return None;
        }

        let verification = self.apply_verification_gate(verification);
        // P0-4: 无验证契约 → 入幻觉桶 (可审计), 同时仍结晶但标记 Unverified。
        if verification.is_none() {
            self.bin_hallucination(
                source_name,
                domain,
                reward,
                "no verification contract (anti-hallucination gate)",
            );
        }
        let description = format!("Auto-crystallized from {} ({})", source_name, domain);
        let crystal_id = self.registry.next_id;
        let pattern = edits.iter().map(|e| format!("{:?}", e)).collect::<Vec<_>>().join(", ");
        let crystal = SkillCrystal {
            id: crystal_id,
            name: description,
            pattern,
            effectiveness: reward,
            use_count: 0,
            source_trace_ids: Vec::new(),
            tags: vec![source_name.to_string(), domain.to_string()],
            strategy: StrategyKind::Reflection,
            domain: AttentionDomain::PatternMatch,
            created_at: 0,
            last_used: 0,
            verification,
        };

        self.registry.crystals.push(crystal.clone());
        self.registry.next_id += 1;

        let mem = ReasoningMemory::new(
            &format!("Crystal: {} from {}", crystal.id, source_url),
            TaskType::General,
            edits,
            reward,
        );
        bank.store(mem);

        self.total_crystallized += 1;

        Some(crystal)
    }

    pub fn crystallize_from_trace(
        &mut self,
        bank: &mut ReasoningBank,
        description: &str,
        insights: &[String],
        confidence: f64,
        verification: Option<VerificationContract>,
    ) -> Option<SkillCrystal> {
        if !self.auto_crystallize || confidence < self.min_reward_threshold {
            return None;
        }

        let verification = self.apply_verification_gate(verification);
        let crystal_id = self.registry.next_id;
        let crystal = SkillCrystal {
            id: crystal_id,
            name: description.to_string(),
            pattern: insights.join("; "),
            effectiveness: confidence,
            use_count: 0,
            source_trace_ids: Vec::new(),
            tags: insights.to_vec(),
            strategy: StrategyKind::Reflection,
            domain: AttentionDomain::PatternMatch,
            created_at: 0,
            last_used: 0,
            verification,
        };

        self.registry.crystals.push(crystal.clone());
        self.registry.next_id += 1;

        let mem = ReasoningMemory::new(
            &format!("Crystal: {} (trace)", crystal.id),
            TaskType::General,
            &[],
            confidence,
        );
        bank.store(mem);

        self.total_crystallized += 1;

        Some(crystal)
    }

    pub fn summary(&self) -> String {
        format!(
            "AutoCrystallizer: {} crystals | auto={} | threshold={:.2} | unverified={} (gate={})",
            self.total_crystallized,
            self.auto_crystallize,
            self.min_reward_threshold,
            self.unverified_crystallized,
            self.anti_hallucination_gate,
        )
    }
}

// ========== P1-17 book-to-skill 蒸馏五件套 (吸收 book-to-skill 模式) ==========
// 大知识源 (书/长文) 蒸馏为: SKILL.md + 章节 + glossary + patterns + cheatsheet。
// 落盘格式强化 — external-absorption 产物从"单条结晶"升级为"可复用技能套件"。

/// 蒸馏章节
#[derive(Debug, Clone)]
pub struct DistilledChapter {
    pub title: String,
    pub body: String,
}

/// 术语表条目
#[derive(Debug, Clone)]
pub struct GlossaryTerm {
    pub term: String,
    pub definition: String,
}

/// 模式条目
#[derive(Debug, Clone)]
pub struct PatternEntry {
    pub name: String,
    pub description: String,
}

/// book-to-skill 蒸馏五件套
#[derive(Debug, Clone)]
pub struct DistillationSuite {
    /// SKILL.md 主文件 — 技能入口 (标题 + 摘要 + 章节索引)
    pub skill_md: String,
    /// 章节 — 按 markdown 标题切分
    pub chapters: Vec<DistilledChapter>,
    /// glossary — 术语表 (**term**: definition 或 - term: definition)
    pub glossary: Vec<GlossaryTerm>,
    /// patterns — 模式库 (Pattern:/模式: 前缀行)
    pub patterns: Vec<PatternEntry>,
    /// cheatsheet — 速查条目 (列表项/代码块)
    pub cheatsheet: Vec<String>,
}

impl DistillationSuite {
    /// 从原始 markdown 知识源蒸馏五件套。
    /// 解析规则:
    ///   - `#`/`##` 标题 → 章节
    ///   - `**term**: def` 或 `- term: def` → glossary
    ///   - `Pattern:` / `模式:` 前缀 → patterns
    ///   - `- ` 列表项 / ``` 代码块 → cheatsheet
    pub fn distill(source_title: &str, raw: &str) -> Self {
        let mut chapters = Vec::new();
        let mut glossary = Vec::new();
        let mut patterns = Vec::new();
        let mut cheatsheet = Vec::new();

        let mut cur_title = String::new();
        let mut cur_body = String::new();
        let mut in_code = false;

        let flush = |cur_title: &mut String, cur_body: &mut String, chapters: &mut Vec<DistilledChapter>| {
            // 章节只要有标题就入册 — 正文可能仅含术语/模式/速查行
            // (这些行被 continue 掉, 不累积进 body), body 空不等于章节不存在。
            if !cur_title.is_empty() {
                chapters.push(DistilledChapter {
                    title: std::mem::take(cur_title),
                    body: std::mem::take(cur_body),
                });
            } else {
                cur_title.clear();
                cur_body.clear();
            }
        };

        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("```") {
                in_code = !in_code;
                if in_code {
                    cheatsheet.push(format!("```{}", &trimmed[3..]));
                } else {
                    cheatsheet.push("```".to_string());
                }
                continue;
            }
            if in_code {
                cheatsheet.push(line.to_string());
                continue;
            }

            // 章节: `##` 及以上 (二级标题) — 一级 `#` 视为文档标题, 不入章节。
            if trimmed.starts_with("##") {
                flush(&mut cur_title, &mut cur_body, &mut chapters);
                cur_title = trimmed.trim_start_matches('#').trim().to_string();
                continue;
            }
            if trimmed.starts_with('#') {
                // 一级标题: 文档主标题 — 作为分隔但自身不入章节。
                flush(&mut cur_title, &mut cur_body, &mut chapters);
                continue;
            }

            // glossary: **term**: definition
            if let Some(rest) = trimmed.strip_prefix("**") {
                if let Some(eq) = rest.find("**:") {
                    let term = rest[..eq].trim().to_string();
                    let definition = rest[eq + 3..].trim().to_string();
                    if !term.is_empty() && !definition.is_empty() {
                        glossary.push(GlossaryTerm { term, definition });
                        continue;
                    }
                }
            }
            // glossary: - term: definition
            if let Some(rest) = trimmed.strip_prefix("- ") {
                if let Some(eq) = rest.find(':') {
                    let term = rest[..eq].trim().to_string();
                    let definition = rest[eq + 1..].trim().to_string();
                    if !term.is_empty() && !definition.is_empty()
                        && !term.starts_with("Pattern") && !term.starts_with("模式") {
                        glossary.push(GlossaryTerm { term, definition });
                        continue;
                    }
                }
            }
            // patterns: Pattern: / 模式: 前缀
            if let Some(rest) = trimmed.strip_prefix("Pattern:") {
                patterns.push(PatternEntry {
                    name: rest.trim().to_string(),
                    description: String::new(),
                });
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("模式:") {
                patterns.push(PatternEntry {
                    name: rest.trim().to_string(),
                    description: String::new(),
                });
                continue;
            }
            // cheatsheet: 列表项
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                cheatsheet.push(trimmed.to_string());
                continue;
            }

            if !cur_title.is_empty() {
                cur_body.push_str(line);
                cur_body.push('\n');
            }
        }
        flush(&mut cur_title, &mut cur_body, &mut chapters);

        // SKILL.md 主文件: 标题 + 摘要 + 章节索引 + 术语/模式/速查计数
        let mut skill_md = format!("# {}\n\n> 蒸馏自外部知识源 (book-to-skill 五件套)\n\n## Chapters\n\n", source_title);
        for (i, ch) in chapters.iter().enumerate() {
            skill_md.push_str(&format!("{}. {}\n", i + 1, ch.title));
        }
        skill_md.push_str(&format!(
            "\n## Stats\n\n- chapters: {}\n- glossary: {}\n- patterns: {}\n- cheatsheet: {}\n",
            chapters.len(), glossary.len(), patterns.len(), cheatsheet.len()
        ));

        Self {
            skill_md,
            chapters,
            glossary,
            patterns,
            cheatsheet,
        }
    }

    /// 五件套完整性检查 — 空套件不可用 (Dark Forest: 无产出即删除)
    pub fn is_usable(&self) -> bool {
        !self.chapters.is_empty() || !self.glossary.is_empty()
            || !self.patterns.is_empty() || !self.cheatsheet.is_empty()
    }
}

impl Default for AutoCrystallizer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_self::skill_crystal::VerificationStatus;

    #[test]
    fn test_new_crystallizer() {
        let c = AutoCrystallizer::new();
        assert!(c.auto_crystallize);
        assert_eq!(c.total_crystallized, 0);
    }

    #[test]
    fn test_crystallize_from_absorption_low_reward() {
        let mut c = AutoCrystallizer::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "https://example.com", "test", "general",
            &[], 0.1, None,
        );
        assert!(result.is_none());
        assert_eq!(c.total_crystallized, 0);
    }

    #[test]
    fn test_crystallize_from_absorption_high_reward() {
        let mut c = AutoCrystallizer::new();
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let edits = vec![
            MicroEdit::AdjustDimension("compound_composition".to_string(), 0.1),
            MicroEdit::NormalizeVector,
        ];
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "https://example.com", "test", "general",
            &edits, 0.8, None,
        );
        assert!(result.is_some());
        assert_eq!(c.total_crystallized, 1);
        let crystal = result.unwrap();
        assert!(crystal.tags.len() <= 2);
    }

    #[test]
    fn test_crystallize_from_trace() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let insights = vec!["pattern: use agent isolation".to_string(), "pattern: module boundaries".to_string()];
        let result = c.crystallize_from_trace(&mut bank, "agent design", &insights, 0.7, None);
        assert!(result.is_some());
        assert_eq!(c.total_crystallized, 1);
    }

    #[test]
    fn test_crystallize_from_trace_low_confidence() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_trace(&mut bank, "test", &[], 0.1, None);
        assert!(result.is_none());
    }

    #[test]
    fn test_crystallizer_disabled() {
        let mut c = AutoCrystallizer::new();
        c.auto_crystallize = false;
        let mut brain = ReasoningBrain::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_absorption(
            &mut brain, &mut bank,
            "url", "name", "domain", &[], 0.9, None,
        );
        assert!(result.is_none());
    }

    // ── P0-1 反幻觉门测试 (DeepZero assessment.j2 吸收) ──

    #[test]
    fn test_anti_hallucination_gate_marks_unverified() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_trace(&mut bank, "no verification", &["insight".to_string()], 0.9, None);
        assert!(result.is_some());
        let crystal = result.unwrap();
        // 无验证契约 → 反幻觉门标记 Unverified 并计数
        assert!(crystal.verification.is_none());
        assert_eq!(c.unverified_crystallized, 1);
        assert!(c.summary().contains("unverified=1"));
    }

    #[test]
    fn test_anti_hallucination_gate_accepts_contract() {
        let mut c = AutoCrystallizer::new();
        let mut bank = ReasoningBank::new(100);
        let contract = VerificationContract {
            observable: "output buffer non-zero and matches memory caller never supplied".to_string(),
            method: "replay trace and check behavior change".to_string(),
            status: VerificationStatus::Pending,
        };
        let result = c.crystallize_from_trace(
            &mut bank, "verified insight", &["insight".to_string()], 0.9, Some(contract),
        );
        assert!(result.is_some());
        let crystal = result.unwrap();
        assert!(crystal.verification.is_some());
        let v = crystal.verification.unwrap();
        assert_eq!(v.status, VerificationStatus::Pending);
        assert_eq!(c.unverified_crystallized, 0);
    }

    #[test]
    fn test_anti_hallucination_gate_can_be_disabled() {
        let mut c = AutoCrystallizer::new();
        c.anti_hallucination_gate = false;
        let mut bank = ReasoningBank::new(100);
        let result = c.crystallize_from_trace(&mut bank, "gate off", &["insight".to_string()], 0.9, None);
        assert!(result.is_some());
        assert_eq!(c.unverified_crystallized, 0);
    }

    #[test]
    fn test_verification_status_roundtrip() {
        // 认识论声明：Confirmed/Refuted 状态可序列化往返
        let contract = VerificationContract {
            observable: "observable".to_string(),
            method: "method".to_string(),
            status: VerificationStatus::Confirmed,
        };
        let json = serde_json::to_string(&contract).unwrap();
        let back: VerificationContract = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, VerificationStatus::Confirmed);
    }

    // ── P1-17 book-to-skill 蒸馏五件套测试 ──

    #[test]
    fn test_distill_extracts_all_five_parts() {
        let raw = "# Agent Design\n\n## Isolation\n\n**Agent**: 独立执行单元\n**Reward**: 反馈信号\n\nPattern: isolate agents per task\n\n- use sandbox per agent\n- cap retries at 3\n\n## Memory\n\n**KB**: 知识库\n\n模式: snapshot before mutation\n";
        let suite = DistillationSuite::distill("Agent Design", raw);
        assert!(suite.is_usable());
        assert_eq!(suite.chapters.len(), 2, "两个章节: {:?}", suite.chapters.iter().map(|c| &c.title).collect::<Vec<_>>());
        assert_eq!(suite.glossary.len(), 3, "三个术语: {:?}", suite.glossary);
        assert_eq!(suite.patterns.len(), 2, "两个模式");
        assert!(suite.cheatsheet.len() >= 2, "速查条目");
        assert!(suite.skill_md.contains("## Chapters"));
        assert!(suite.skill_md.contains("1. Isolation"));
        assert!(suite.skill_md.contains("2. Memory"));
    }

    #[test]
    fn test_distill_empty_source_not_usable() {
        let suite = DistillationSuite::distill("Empty", "");
        assert!(!suite.is_usable(), "空源不可用 (Dark Forest)");
        assert!(suite.chapters.is_empty());
    }

    #[test]
    fn test_distill_code_block_goes_to_cheatsheet() {
        let raw = "# Tool\n\n## Usage\n\n```rust\nlet x = 1;\n```\n";
        let suite = DistillationSuite::distill("Tool", raw);
        assert!(suite.cheatsheet.iter().any(|c| c.contains("let x = 1")),
            "代码块应入 cheatsheet: {:?}", suite.cheatsheet);
    }
}
