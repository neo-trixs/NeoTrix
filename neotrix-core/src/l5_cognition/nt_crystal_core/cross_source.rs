//! 跨源融合引擎 — KB知识 + 经验 + 外部数据 → 晶体进化

use super::CrystalCore;
use super::engine::CrystalEngine;
use super::knowledge::{CausalPattern, Contradiction, Counterfactual, Theory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据源类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DataSource {
    /// KB 知识库
    KB,
    /// 会话经验 (pending-absorb)
    Experience,
    /// 外部数据 (crawl/web)
    External,
    /// 融合生成 (cross-source)
    Fused,
}

/// 统一数据条目 — 所有源的数据都归一化为这个格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEntry {
    pub id: String,
    pub source: DataSource,
    pub domain: String,
    pub content: String,
    pub entry_type: EntryType,
    pub confidence: f64,
    pub metadata: HashMap<String, String>,
    pub timestamp: String,
}

/// 条目类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntryType {
    /// 理论/知识
    Knowledge,
    /// 因果模式
    Pattern,
    /// 矛盾
    Contradiction,
    /// 反事实
    Counterfactual,
    /// 经验
    Episode,
    /// 失败教训
    Lesson,
    /// 成功方案
    Solution,
    /// 能力信号
    CapabilitySignal,
}

/// 融合报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusionReport {
    pub kb_entries: usize,
    pub experience_entries: usize,
    pub external_entries: usize,
    pub fused_patterns: usize,
    pub new_contradictions: usize,
    pub new_counterfactuals: usize,
    pub capability_deltas: Vec<(String, f64)>,
    pub total_entries_processed: usize,
}

/// 跨源融合引擎
pub struct CrossSourceFusionEngine;

impl CrossSourceFusionEngine {
    /// 主融合循环 — 吸收所有源 → 跨源模式提取 → 进化
    pub fn run_fusion_cycle(core: &mut CrystalCore) -> Result<FusionReport, String> {
        let mut report = FusionReport {
            kb_entries: 0,
            experience_entries: 0,
            external_entries: 0,
            fused_patterns: 0,
            new_contradictions: 0,
            new_counterfactuals: 0,
            capability_deltas: Vec::new(),
            total_entries_processed: 0,
        };

        // 阶段1: 从各源吸收数据
        let kb_entries = Self::absorb_from_kb(core);
        report.kb_entries = kb_entries;

        let exp_entries = Self::absorb_from_pending(core);
        report.experience_entries = exp_entries;

        let ext_entries = Self::absorb_from_external(core);
        report.external_entries = ext_entries;

        report.total_entries_processed = kb_entries + exp_entries + ext_entries;

        // 阶段2: 跨源模式提取
        let fused = Self::extract_cross_source_patterns(core);
        report.fused_patterns = fused;

        // 阶段3: 矛盾检测
        let contradictions = Self::detect_contradictions(core);
        report.new_contradictions = contradictions;

        // 阶段4: 反事实生成
        let counterfactuals = Self::generate_counterfactuals(core);
        report.new_counterfactuals = counterfactuals;

        // 阶段5: 能力评分更新
        let deltas = Self::update_capability_scores(core);
        report.capability_deltas = deltas;

        // 阶段6: 记录生长周期
        let overall_before = core.evolution.capability_scores.overall();
        core.evolution.record_cycle(
            core.evolution.current_phase.clone(),
            vec![
                format!("Fused {} patterns", report.fused_patterns),
                format!("Detected {} contradictions", report.new_contradictions),
                format!("Generated {} counterfactuals", report.new_counterfactuals),
            ],
            overall_before,
            core.evolution.capability_scores.overall(),
            0,
        );

        // 阶段7: 持久化
        core.save()?;

        Ok(report)
    }

    /// 从 KB 知识库吸收
    fn absorb_from_kb(core: &mut CrystalCore) -> usize {
        let kb_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("knowledge.db");

        if !kb_path.exists() {
            return 0;
        }

        let mut count = 0;

        // 读取 KB 中的 pattern 命名空间
        if let Ok(entries) = Self::read_kb_namespace(&kb_path, "pattern") {
            for entry in entries {
                let pattern = CausalPattern {
                    id: format!("KB-{:04}", core.knowledge.patterns.len() + 1),
                    if_conditions: vec![entry.get("if").cloned().unwrap_or_default()],
                    then_consequences: vec![entry.get("then").cloned().unwrap_or_default()],
                    so_implications: vec![entry.get("so").cloned().unwrap_or_default()],
                    theory_origin: "KB".into(),
                    confidence: 0.6,
                };
                core.knowledge.add_pattern(pattern);
                count += 1;
            }
        }

        // 读取 KB 中的 theory 命名空间
        if let Ok(entries) = Self::read_kb_namespace(&kb_path, "theory") {
            for entry in entries {
                let id = entry.get("id").cloned().unwrap_or_default();
                if !id.is_empty() && !core.knowledge.theories.contains_key(&id) {
                    let theory = Theory {
                        id: id.clone(),
                        name: entry.get("name").cloned().unwrap_or_default(),
                        full_name: entry.get("full_name").cloned().unwrap_or_default(),
                        core_claim: entry.get("claim").cloned().unwrap_or_default(),
                        mathematical_basis: entry.get("math").cloned().unwrap_or_default(),
                        neo_trix_mapping: entry.get("mapping").cloned().unwrap_or_default(),
                        confidence: 0.5,
                    };
                    core.knowledge.add_theory(theory);
                    count += 1;
                }
            }
        }

        count
    }

    /// 从 pending-absorb.json 吸收会话经验
    fn absorb_from_pending(core: &mut CrystalCore) -> usize {
        let pending_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("pending-absorb.json");

        if !pending_path.exists() {
            return 0;
        }

        let content = match std::fs::read_to_string(&pending_path) {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let items = match CrystalEngine::parse_pending_absorb(&content) {
            Ok(i) => i,
            Err(_) => return 0,
        };

        let count = items.len();
        for item in items {
            let _ = CrystalEngine::absorb(
                core,
                &item.context,
                &item.action,
                &item.result,
                &item.reflection,
                &item.domain,
            );
        }

        // 清理 pending 文件
        let _ = std::fs::remove_file(&pending_path);
        count
    }

    /// 从外部数据源吸收 (crawl queue / web sources)
    fn absorb_from_external(core: &mut CrystalCore) -> usize {
        let crawl_queue_path = dirs::home_dir()
            .unwrap_or_default()
            .join(".neotrix")
            .join("crawl_queue.jsonl");

        if !crawl_queue_path.exists() {
            return 0;
        }

        let mut count = 0;
        if let Ok(content) = std::fs::read_to_string(&crawl_queue_path) {
            for line in content.lines().take(100) { // 限制每次最多处理100条
                if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
                    let url = item.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("");
                    let content_text = item.get("content").and_then(|v| v.as_str()).unwrap_or("");
                    let domain = item.get("domain").and_then(|v| v.as_str()).unwrap_or("external");

                    if !content_text.is_empty() {
                        // 将外部内容转为经验记录
                        let _ = CrystalEngine::absorb(
                            core,
                            &format!("External source: {}", url),
                            &format!("Crawled: {}", title),
                            &format!("Content length: {} chars", content_text.len()),
                            &format!("Domain: {}", domain),
                            domain,
                        );
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// 跨源模式提取 — 从不同源的数据中发现共同模式
    pub fn extract_cross_source_patterns(core: &mut CrystalCore) -> usize {
        let mut new_patterns = 0;

        // 收集所有领域
        let domains: Vec<String> = core.knowledge.domain_tags.clone();
        let experiences: Vec<_> = core.experience.episodes.clone();
        let patterns: Vec<_> = core.knowledge.patterns.clone();

        // 跨源模式1: 同一领域在不同源中出现 → 强化该领域的因果链
        for domain in &domains {
            let kb_patterns: Vec<_> = patterns.iter()
                .filter(|p| p.theory_origin == "KB" || p.theory_origin == "Experience")
                .collect();
            let exp_count = experiences.iter()
                .filter(|e| e.domain == *domain)
                .count();

            if kb_patterns.len() > 2 && exp_count > 5 {
                // 该领域在 KB 和经验中都有大量数据 → 生成融合模式
                let fused = CausalPattern {
                    id: format!("FUSED-{:04}", core.knowledge.patterns.len() + 1),
                    if_conditions: vec![
                        format!("Domain {} has {} KB patterns", domain, kb_patterns.len()),
                        format!("Domain {} has {} experiences", domain, exp_count),
                    ],
                    then_consequences: vec![
                        format!("Domain {} is well-understood", domain),
                        format!("Domain {} patterns are validated", domain),
                    ],
                    so_implications: vec![
                        format!("Domain {} is ready for advanced reasoning", domain),
                    ],
                    theory_origin: "CrossSource".into(),
                    confidence: 0.8,
                };
                core.knowledge.add_pattern(fused);
                new_patterns += 1;
            }
        }

        // 跨源模式2: 失败+成功 → 因果链
        for lesson in &core.experience.failures {
            for solution in &core.experience.successes {
                if lesson.category == solution.domain {
                    let fused = CausalPattern {
                        id: format!("FUSED-{:04}", core.knowledge.patterns.len() + 1),
                        if_conditions: vec![
                            format!("Failure in {}: {}", lesson.category, lesson.failure_description),
                            format!("Success in {}: {}", solution.domain, solution.approach),
                        ],
                        then_consequences: vec![
                            format!("Lesson learned: {}", lesson.takeaway),
                            format!("Reusable pattern: {}", solution.pattern),
                        ],
                        so_implications: vec![
                            format!("Combine fix '{}' with pattern '{}'", lesson.fix_applied, solution.pattern),
                        ],
                        theory_origin: "CrossSource".into(),
                        confidence: 0.75,
                    };
                    core.knowledge.add_pattern(fused);
                    new_patterns += 1;
                }
            }
        }

        // 跨源模式3: 理论+经验 → 理论验证
        let theory_ids: Vec<_> = core.knowledge.theories.keys().cloned().collect();
        for id in &theory_ids {
            let related_experiences: Vec<_> = core.experience.episodes.iter()
                .filter(|e| e.domain.to_lowercase().contains(&id.to_lowercase()))
                .collect();

            if related_experiences.len() > 3 {
                let avg_quality: f64 = related_experiences.iter()
                    .map(|e| e.quality)
                    .sum::<f64>() / related_experiences.len() as f64;

                // 理论有经验支持 → 提高置信度
                if avg_quality > 0.7 {
                    if let Some(t) = core.knowledge.theories.get_mut(id) {
                        t.confidence = (t.confidence + 0.05).min(1.0);
                    }
                }
            }
        }

        new_patterns
    }

    /// 矛盾检测 — 发现知识/经验中的矛盾
    pub fn detect_contradictions(core: &mut CrystalCore) -> usize {
        let mut new_contradictions = 0;

        // 检测1: 理论间的矛盾
        let theory_ids: Vec<String> = core.knowledge.theories.keys().cloned().collect();
        for i in 0..theory_ids.len() {
            for j in (i+1)..theory_ids.len() {
                let t1 = &core.knowledge.theories[&theory_ids[i]];
                let t2 = &core.knowledge.theories[&theory_ids[j]];

                // 如果两个理论的 core_claim 包含相反的关键词
                if Self::claims_contradict(&t1.core_claim, &t2.core_claim) {
                    let contradiction = Contradiction {
                        id: format!("CONTRA-{:04}", core.knowledge.contradictions.len() + 1),
                        thesis: format!("{}: {}", t1.id, t1.core_claim),
                        antithesis: format!("{}: {}", t2.id, t2.core_claim),
                        resolution: format!("Both theories may describe different aspects of consciousness"),
                        category: "theoretical".into(),
                    };
                    core.knowledge.add_contradiction(contradiction);
                    new_contradictions += 1;
                }
            }
        }

        // 检测2: 经验与理论的矛盾
        let theories_clone: Vec<_> = core.knowledge.theories.iter().map(|(k, v)| (k.clone(), v.core_claim.clone())).collect();
        for episode in &core.experience.episodes {
            if episode.quality < 0.3 {
                // 低质量经验可能与理论矛盾
                for (id, claim) in &theories_clone {
                    if episode.domain.to_lowercase().contains(&id.to_lowercase()) {
                        let contradiction = Contradiction {
                            id: format!("CONTRA-{:04}", core.knowledge.contradictions.len() + 1),
                            thesis: format!("Theory {}: {}", id, claim),
                            antithesis: format!("Low-quality experience in {}: {}", episode.domain, episode.reflection),
                            resolution: "Experience may indicate theory limitation or measurement error".into(),
                            category: "empirical".into(),
                        };
                        core.knowledge.add_contradiction(contradiction);
                        new_contradictions += 1;
                    }
                }
            }
        }

        new_contradictions
    }

    /// 反事实生成 — 基于当前知识生成反事实
    pub fn generate_counterfactuals(core: &mut CrystalCore) -> usize {
        let mut new_counterfactuals = 0;

        // 基于能力差距生成反事实
        let gaps = core.evolution.capability_scores.gaps(0.5);
        for (capability, score) in &gaps {
            let cf = Counterfactual {
                id: format!("CF-{:04}", core.knowledge.counterfactuals.len() + 1),
                if_condition: format!("Capability '{}' was at 0.9 instead of {:.2}", capability, score),
                then_consequence: format!("System could handle {} tasks 3x faster", capability),
                so_implication: format!("Priority: invest in improving {}", capability),
                domain: capability.clone(),
            };
            core.knowledge.counterfactuals.push(cf);
            new_counterfactuals += 1;
        }

        // 基于失败生成反事实
        let failure_lessons: Vec<_> = core.experience.failures.iter().take(5).cloned().collect();
        for lesson in &failure_lessons {
            let cf = Counterfactual {
                id: format!("CF-{:04}", core.knowledge.counterfactuals.len() + 1),
                if_condition: format!("Failure '{}' was prevented", lesson.failure_description),
                then_consequence: format!("Would have saved effort on fix: {}", lesson.fix_applied),
                so_implication: format!("Prevention rule: {}", lesson.takeaway),
                domain: lesson.category.clone(),
            };
            core.knowledge.counterfactuals.push(cf);
            new_counterfactuals += 1;
        }

        new_counterfactuals
    }

    /// 更新能力评分
    pub fn update_capability_scores(core: &mut CrystalCore) -> Vec<(String, f64)> {
        let mut deltas = Vec::new();

        // 基于经验数量更新
        let ep_count = core.experience.episodes.len();
        if ep_count > 10 {
            let current = core.evolution.capability_scores.get("reasoning");
            let new_val = (current + 0.01).min(1.0);
            core.evolution.update_score("reasoning", new_val);
            deltas.push(("reasoning".into(), new_val - current));
        }

        // 基于失败数量更新
        let fail_count = core.experience.failures.len();
        if fail_count > 5 {
            let current = core.evolution.capability_scores.get("safety");
            let new_val = (current + 0.005).min(1.0);
            core.evolution.update_score("safety", new_val);
            deltas.push(("safety".into(), new_val - current));
        }

        // 基于成功方案数量更新
        let sol_count = core.experience.successes.len();
        if sol_count > 3 {
            let current = core.evolution.capability_scores.get("creativity");
            let new_val = (current + 0.01).min(1.0);
            core.evolution.update_score("creativity", new_val);
            deltas.push(("creativity".into(), new_val - current));
        }

        // 基于知识库大小更新
        let theory_count = core.knowledge.theories.len();
        let pattern_count = core.knowledge.patterns.len();
        if theory_count > 3 && pattern_count > 10 {
            let current = core.evolution.capability_scores.get("memory");
            let new_val = (current + 0.01).min(1.0);
            core.evolution.update_score("memory", new_val);
            deltas.push(("memory".into(), new_val - current));
        }

        // 基于融合模式更新
        let fused_count = core.knowledge.patterns.iter()
            .filter(|p| p.theory_origin == "CrossSource")
            .count();
        if fused_count > 5 {
            let current = core.evolution.capability_scores.get("evolution");
            let new_val = (current + 0.02).min(1.0);
            core.evolution.update_score("evolution", new_val);
            deltas.push(("evolution".into(), new_val - current));
        }

        deltas
    }

    /// 读取 KB 命名空间 (简化版 — 从 JSONL 文件读取)
    fn read_kb_namespace(kb_path: &std::path::Path, namespace: &str) -> Result<Vec<HashMap<String, String>>, String> {
        let ns_path = kb_path.parent()
            .unwrap_or(kb_path)
            .join(format!("kb_{}.jsonl", namespace));

        if !ns_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&ns_path)
            .map_err(|e| format!("Failed to read KB namespace: {}", e))?;

        let mut entries = Vec::new();
        for line in content.lines() {
            if let Ok(item) = serde_json::from_str::<serde_json::Value>(line) {
                let mut map = HashMap::new();
                if let Some(obj) = item.as_object() {
                    for (k, v) in obj {
                        if let Some(s) = v.as_str() {
                            map.insert(k.clone(), s.to_string());
                        }
                    }
                }
                entries.push(map);
            }
        }

        Ok(entries)
    }

    /// 简单的声称矛盾检测
    fn claims_contradict(claim1: &str, claim2: &str) -> bool {
        let contradicting_pairs = [
            ("global", "local"),
            ("broadcast", "private"),
            ("integrated", "segregated"),
            ("conscious", "unconscious"),
            ("deterministic", "random"),
            ("reductionist", "holistic"),
        ];

        let c1 = claim1.to_lowercase();
        let c2 = claim2.to_lowercase();

        for (a, b) in contradicting_pairs {
            if (c1.contains(a) && c2.contains(b)) || (c1.contains(b) && c2.contains(a)) {
                return true;
            }
        }

        false
    }
}
