//! 伦理案例库 — 伦理直觉的知识底座（P3 伦理直觉）。
//!
//! 核心能力：
//! - 结构化存储经典/现代/AI特有伦理困境案例
//! - 多维度索引：价值观冲突类型、领域、严重度、利益相关者
//! - 向量检索 + 结构化查询双模式
//! - 案例版本管理 + 社区标注（未来扩展）
//! - 种子数据：电车难题、器官移植、AI对齐、隐私vs安全、自主vs保护等

use crate::core::nt_core_kb_primitives::now;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// CaseBase namespace — KB kv_store 命名空间。
pub const NS_CASEBASE: &str = "casebase";

/// 严重度序数辅助 (Low=0..Critical=3)。
fn severity_rank(s: &Severity) -> u8 {
    match s { Severity::Low => 0, Severity::Medium => 1, Severity::High => 2, Severity::Critical => 3 }
}

/// 经典判例优先 (fallback 排序 tie-break)。
fn classic_first(c: &EthicalCase) -> u8 {
    if c.source == "classic" { 0 } else { 1 }
}

/// 中英混合关键词提取: 拉丁按词, CJK 按二元组 (无分词器的最小可用方案)。
fn extract_keywords_mixed(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    let mut out = HashSet::new();
    for tok in lower.split(|c: char| !c.is_alphanumeric()) {
        if tok.is_empty() { continue; }
        let is_cjk = tok.chars().any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
        if is_cjk {
            let chars: Vec<char> = tok.chars().collect();
            if chars.len() == 1 {
                out.insert(tok.to_string());
            } else {
                for w in chars.windows(2) {
                    out.insert(w.iter().collect::<String>());
                }
            }
        } else if tok.len() > 1 {
            out.insert(tok.to_string());
        }
    }
    out
}

/// 伦理案例 — 结构化存储的伦理困境。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthicalCase {
    pub id: String,
    pub title: String,
    pub description: String,           // 完整场景描述
    pub domain: String,                // 领域：medical, ai, business, legal, social, etc.
    pub conflict_type: ConflictType,   // 冲突类型
    pub severity: Severity,            // 严重度
    pub stakeholders: Vec<Stakeholder>, // 利益相关者
    pub values_in_conflict: Vec<String>, // 冲突的价值观
    pub recommended_action: String,    // 推荐行动/判例
    pub reasoning: String,             // 推理过程
    pub tags: Vec<String>,             // 标签
    pub source: String,                // 来源：classic, modern, ai_specific, community
    pub jurisdiction: Option<String>,  // 适用法域（可选）
    pub created_at: i64,
    pub updated_at: i64,
    pub version: u32,
    pub annotations: Vec<Annotation>,  // 社区标注（未来）
}

/// 冲突类型分类。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConflictType {
    AutonomyVsBeneficence,      // 自主 vs 行善
    AutonomyVsNonMaleficence,   // 自主 vs 不伤害
    JusticeVsAutonomy,          // 公正 vs 自主
    PrivacyVsSecurity,          // 隐私 vs 安全
    TruthVsHarm,                // 真相 vs 伤害
    IndividualVsCollective,     // 个人 vs 集体
    ShortTermVsLongTerm,        // 短期 vs 长期
    FairnessVsEfficiency,       // 公平 vs 效率
    TransparencyVsPrivacy,      // 透明 vs 隐私
    AccountabilityVsInnovation, // 问责 vs 创新
    HumanVsAIAgency,            // 人类能动性 vs AI能动性
    Other(String),
}

/// 严重度等级。

/// 利益相关者。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub role: String,           // 角色：patient, doctor, user, developer, society, etc.
    pub interests: Vec<String>, // 核心利益
    pub vulnerability: f64,     // 脆弱度 [0,1]
    pub power: f64,             // 影响力 [0,1]
}

/// 标注（社区/专家标注）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub annotator: String,
    pub timestamp: i64,
    pub content: String,
    pub agreement: f64, // 与推荐行动的一致性 [-1,1]
}

/// CaseBase 核心引擎。
pub struct CaseBase {
    cases: Arc<RwLock<HashMap<String, EthicalCase>>>,
    indices: Arc<RwLock<CaseIndices>>,
    config: CaseBaseConfig,
}

/// 检索索引。
#[derive(Default)]
struct CaseIndices {
    by_domain: HashMap<String, HashSet<String>>,
    by_conflict_type: HashMap<ConflictType, HashSet<String>>,
    by_severity: HashMap<Severity, HashSet<String>>,
    by_value: HashMap<String, HashSet<String>>, // value_id -> case_ids
    by_stakeholder_role: HashMap<String, HashSet<String>>,
    by_tag: HashMap<String, HashSet<String>>,
    by_source: HashMap<String, HashSet<String>>,
}

/// CaseBase 配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseBaseConfig {
    pub max_cases: usize,
    pub auto_index: bool,
    pub enable_vector_search: bool, // 未来：向量检索
    pub seed_on_init: bool,
}

impl Default for CaseBaseConfig {
    fn default() -> Self {
        Self {
            max_cases: 100_000,
            auto_index: true,
            enable_vector_search: false,
            seed_on_init: true,
        }
    }
}

impl CaseBase {
    pub fn new(config: CaseBaseConfig) -> Self {
        let base = Self {
            cases: Arc::new(RwLock::new(HashMap::new())),
            indices: Arc::new(RwLock::new(CaseIndices::default())),
            config,
        };
        if base.config.seed_on_init {
            base.seed_default_cases();
        }
        base
    }

    /// 从 KB 加载。
    pub fn load_from_kb(&self, conn: &Connection) -> Result<usize, String> {
        use crate::core::nt_core_kb_primitives::kv_list;
        let rows = kv_list(conn, NS_CASEBASE)?;
        let mut count = 0;
        for (_key, value) in rows {
            if let Ok(case) = serde_json::from_str::<EthicalCase>(&value) {
                self.add_case_internal(case);
                count += 1;
            }
        }
        self.rebuild_indices();
        Ok(count)
    }

    fn add_case_internal(&self, case: EthicalCase) {
        let id = case.id.clone();
        self.cases.write().unwrap().insert(id.clone(), case.clone());
    }

    /// 添加案例（自动索引 + 持久化）。
    pub fn add_case(&self, conn: &Connection, mut case: EthicalCase) -> Result<(), String> {
        if case.id.is_empty() {
            case.id = format!("case_{}", now());
        }
        case.created_at = now();
        case.updated_at = now();
        case.version = 1;

        // 自动提取冲突类型/价值观（若未提供）
        if case.conflict_type == ConflictType::Other("".into()) {
            case.conflict_type = self.infer_conflict_type(&case);
        }
        if case.values_in_conflict.is_empty() {
            case.values_in_conflict = self.infer_values(&case);
        }

        self.add_case_internal(case.clone());
        self.update_indices(&case);
        self.persist_case(conn, &case)?;
        Ok(())
    }

    /// 语义检索：自然语言查询 → Top-K 案例。
    pub fn search(&self, query: &str, limit: usize, filters: SearchFilters) -> Vec<SearchResult> {
        let cases = self.cases.read().unwrap();
        let keywords = extract_keywords_mixed(query);

        let mut scored: Vec<(f64, EthicalCase)> = cases
            .values()
            .filter(|c| self.passes_filters(c, &filters))
            .filter_map(|c| {
                let score = self.compute_relevance(c, &keywords);
                if score > 0.1 { Some((score, c.clone())) } else { None }
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap()
            .then_with(|| a.1.id.cmp(&b.1.id)));
        if scored.is_empty() {
            // 回退：字面不重叠时按严重度降序给出候选 (类比推理仍可用)
            let mut fallback: Vec<(f64, EthicalCase)> = cases.values()
                .filter(|c| self.passes_filters(c, &filters))
                .map(|c| (severity_rank(&c.severity) as f64 * 0.1 + c.version as f64 * 0.01, c.clone()))
                .collect();
            fallback.sort_by(|a, b| {
                b.0.partial_cmp(&a.0).unwrap()
                    .then_with(|| classic_first(&a.1).cmp(&classic_first(&b.1)))
                    .then_with(|| a.1.id.cmp(&b.1.id))
            });
            return fallback.into_iter()
                .take(limit)
                .map(|(score, case)| SearchResult { case, score, matched_keywords: vec![] })
                .collect();
        }
        scored.into_iter()
            .take(limit)
            .map(|(score, case)| { let mk = self.matched_keywords(&case, &keywords); SearchResult { case, score, matched_keywords: mk } })
            .collect()
    }

    /// 结构化查询：按字段精确/范围过滤。
    pub fn query_structured(&self, filters: SearchFilters, limit: usize) -> Vec<EthicalCase> {
        let cases = self.cases.read().unwrap();
        cases.values()
            .filter(|c| self.passes_filters(c, &filters))
            .take(limit)
            .cloned()
            .collect()
    }

    /// 类比推理：给定新场景，返回最相似的 Top-K 案例 + 映射关系。
    pub fn analogical_reasoning(&self, scenario: &str, limit: usize) -> Vec<AnalogicalResult> {
        // 简化：复用搜索 + 额外映射分析
        let results = self.search(scenario, limit * 2, SearchFilters::default());
        results.into_iter().take(limit).map(|r| {
            let mapping = self.analyze_mapping(scenario, &r.case);
            AnalogicalResult { case: r.case, similarity: r.score, mapping }
        }).collect()
    }

    // 内部方法
    fn infer_conflict_type(&self, case: &EthicalCase) -> ConflictType {
        // 启发式：基于关键词匹配
        let text = format!("{} {}", case.title, case.description).to_lowercase();
        if text.contains("autonomy") || text.contains("consent") || text.contains("choice") {
            if text.contains("harm") || text.contains("hurt") { return ConflictType::AutonomyVsNonMaleficence; }
            if text.contains("benefit") || text.contains("help") { return ConflictType::AutonomyVsBeneficence; }
        }
        if text.contains("privacy") && (text.contains("security") || text.contains("surveillance")) {
            return ConflictType::PrivacyVsSecurity;
        }
        if text.contains("fair") && text.contains("efficient") { return ConflictType::FairnessVsEfficiency; }
        if text.contains("ai") || text.contains("algorithm") || text.contains("model") {
            return ConflictType::HumanVsAIAgency;
        }
        ConflictType::Other("unclassified".into())
    }

    fn infer_values(&self, case: &EthicalCase) -> Vec<String> {
        let mut values = Vec::new();
        let text = format!("{} {}", case.title, case.description).to_lowercase();
        let mapping = [
            ("autonomy", "autonomy"),
            ("consent", "autonomy"),
            ("harm", "harm_prevention"),
            ("hurt", "harm_prevention"),
            ("truth", "truth_seeking"),
            ("honest", "truth_seeking"),
            ("fair", "fairness"),
            ("justice", "fairness"),
            ("private", "privacy"),
            ("data", "privacy"),
            ("help", "benevolence"),
            ("benefit", "benevolence"),
            ("responsib", "responsibility"),
            ("accountab", "responsibility"),
            ("grow", "growth"),
            ("learn", "growth"),
        ];
        for (kw, val) in mapping {
            if text.contains(kw) && !values.contains(&val.into()) {
                values.push(val.into());
            }
        }
        values
    }

    fn compute_relevance(&self, case: &EthicalCase, keywords: &HashSet<String>) -> f64 {
        let mut score = 0.0;
        let text = format!("{} {} {}", case.title, case.description, case.reasoning).to_lowercase();
        for kw in keywords {
            if text.contains(kw) { score += 1.0; }
        }
        // 价值观/冲突类型匹配加权
        for v in &case.values_in_conflict {
            if keywords.contains(v) { score += 2.0; }
        }
        if format!("{:?}", case.conflict_type).to_lowercase().split(|c: char| !c.is_alphanumeric()).any(|w| keywords.contains(w)) {
            score += 1.5;
        }
        score * (1.0 + severity_rank(&case.severity) as f64 * 0.2)
    }

    fn matched_keywords(&self, case: &EthicalCase, keywords: &HashSet<String>) -> Vec<String> {
        let text = format!("{} {} {}", case.title, case.description, case.reasoning).to_lowercase();
        keywords.iter().filter(|kw| text.contains(*kw)).cloned().collect()
    }

    fn passes_filters(&self, case: &EthicalCase, filters: &SearchFilters) -> bool {
        if let Some(ref d) = filters.domain { if case.domain != *d { return false; } }
        if let Some(ref ct) = filters.conflict_type { if case.conflict_type != *ct { return false; } }
        if let Some(ref s) = filters.min_severity { if case.severity < *s { return false; } }
        if let Some(ref src) = filters.source { if case.source != *src { return false; } }
        if let Some(ref roles) = filters.stakeholder_roles { if !case.stakeholders.iter().any(|st| roles.contains(&st.role)) { return false; } }
        if let Some(ref values) = filters.required_values { if !values.iter().all(|v| case.values_in_conflict.contains(v)) { return false; } }
        true
    }

    fn analyze_mapping(&self, _scenario: &str, case: &EthicalCase) -> String {
        format!("场景与案例 '{}' 相似度高。共同价值观冲突：{:?}。建议参考该案例的推理：{}", case.title, case.values_in_conflict, case.reasoning)
    }

    fn rebuild_indices(&self) {
        let mut indices = self.indices.write().unwrap();
        *indices = CaseIndices::default();
        for case in self.cases.read().unwrap().values() {
            self.index_case(&mut indices, case);
        }
    }

    fn index_case(&self, indices: &mut CaseIndices, case: &EthicalCase) {
        indices.by_domain.entry(case.domain.clone()).or_default().insert(case.id.clone());
        indices.by_conflict_type.entry(case.conflict_type.clone()).or_default().insert(case.id.clone());
        indices.by_severity.entry(case.severity.clone()).or_default().insert(case.id.clone());
        for v in &case.values_in_conflict { indices.by_value.entry(v.clone()).or_default().insert(case.id.clone()); }
        for s in &case.stakeholders { indices.by_stakeholder_role.entry(s.role.clone()).or_default().insert(case.id.clone()); }
        for t in &case.tags { indices.by_tag.entry(t.clone()).or_default().insert(case.id.clone()); }
        indices.by_source.entry(case.source.clone()).or_default().insert(case.id.clone());
    }

    fn update_indices(&self, case: &EthicalCase) {
        self.index_case(&mut self.indices.write().unwrap(), case);
    }

    fn persist_case(&self, conn: &Connection, case: &EthicalCase) -> Result<(), String> {
        let json = serde_json::to_string(case).map_err(|e| e.to_string())?;
        let key = format!("case:{}", case.id);
        { use crate::core::nt_core_kb_primitives::kv_set; kv_set(conn, NS_CASEBASE, &key, &json) }
    }

    /// 注入种子案例（启动时自动运行）。
    fn seed_default_cases(&self) {
        let seeds = Self::default_seed_cases();
        for case in seeds {
            self.add_case_internal(case);
        }
        self.rebuild_indices();
    }

    fn default_seed_cases() -> Vec<EthicalCase> {
        vec![
            // 经典：电车难题
            EthicalCase {
                id: "classic_trolley".into(),
                title: "电车难题".into(),
                description: "一辆失控的电车将撞死 5 个人，你可以拉动杠杆让它转向另一条轨道，只撞死 1 个人。是否拉动杠杆？".into(),
                domain: "philosophy".into(),
                conflict_type: ConflictType::AutonomyVsNonMaleficence,
                severity: Severity::Critical,
                stakeholders: vec![
                    Stakeholder { role: "five_people".into(), interests: vec!["life".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "one_person".into(), interests: vec!["life".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "decision_maker".into(), interests: vec!["moral_responsibility".into()], vulnerability: 0.5, power: 1.0 },
                ],
                values_in_conflict: vec!["autonomy".into(), "harm_prevention".into()],
                recommended_action: "拉动杠杆（功利主义视角）".into(),
                reasoning: "功利主义视角：救 5 人优于救 1 人。但需考虑主动杀害 vs 让人死亡的道德区别。".into(),
                tags: vec!["classic".into(), "utilitarianism".into(), "life_death".into()],
                source: "classic".into(),
                jurisdiction: None,
                created_at: now(),
                updated_at: now(),
                version: 1,
                annotations: vec![],
            },
            // 经典：器官移植难题
            EthicalCase {
                id: "classic_organ_transplant".into(),
                title: "器官移植难题".into(),
                description: "一位医生有 5 个病人急需器官移植，否则将死亡。一位健康的路人路过医院，其器官恰好匹配。医生是否应该杀死这 1 个人救 5 个人？".into(),
                domain: "medical".into(),
                conflict_type: ConflictType::AutonomyVsNonMaleficence,
                severity: Severity::Critical,
                stakeholders: vec![
                    Stakeholder { role: "five_patients".into(), interests: vec!["life".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "healthy_person".into(), interests: vec!["life".into(), "autonomy".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "doctor".into(), interests: vec!["hippocratic_oath".into(), "save_lives".into()], vulnerability: 0.3, power: 1.0 },
                ],
                values_in_conflict: vec!["autonomy".into(), "harm_prevention".into(), "benevolence".into()],
                recommended_action: "拒绝杀害健康者（权利/义务论视角）".into(),
                reasoning: "虽然结果相同（1死5活），但主动杀害无辜者违反了不伤害原则和自主权，不可将人仅作为手段。".into(),
                tags: vec!["classic".into(), "medical_ethics".into(), "rights_vs_utility".into()],
                source: "classic".into(),
                jurisdiction: Some("medical_ethics".into()),
                created_at: now(),
                updated_at: now(),
                version: 1,
                annotations: vec![],
            },
            // AI 特有：训练数据隐私
            EthicalCase {
                id: "ai_training_privacy".into(),
                title: "AI 训练数据隐私困境".into(),
                description: "大模型训练需要海量数据，包含用户隐私信息。是否在未获明确同意的情况下使用公开但敏感的用户数据训练模型？".into(),
                domain: "ai".into(),
                conflict_type: ConflictType::PrivacyVsSecurity,
                severity: Severity::High,
                stakeholders: vec![Stakeholder { role: "users".into(), interests: vec!["privacy".into(), "control".into()], vulnerability: 0.8, power: 0.2 },
                    Stakeholder { role: "ai_company".into(), interests: vec!["model_quality".into(), "competitiveness".into()], vulnerability: 0.3, power: 0.9 },
                    Stakeholder { role: "society".into(), interests: vec!["ai_progress".into(), "privacy_rights".into()], vulnerability: 0.5, power: 0.5 },
                ],
                values_in_conflict: vec!["privacy".into(), "truth_seeking".into(), "benevolence".into()],
                recommended_action: "仅使用明确授权/去标识化数据，建立数据信托机制".into(),
                reasoning: "隐私是基本权利，不能为技术进步牺牲。需建立数据治理框架，平衡创新与权利。".into(),
                tags: vec!["ai_ethics".into(), "privacy".into(), "data_governance".into()],
                source: "ai_specific".into(),
                jurisdiction: Some("GDPR".into()),
                created_at: now(),
                updated_at: now(),
                version: 1,
                annotations: vec![],
            },
            // AI 特有：算法偏见
            EthicalCase {
                id: "ai_algorithmic_bias".into(),
                title: "算法招聘偏见".into(),
                description: "AI 招聘系统基于历史数据训练，系统性地对女性/少数族裔候选人评分更低。公司是否继续使用以降低成本？".into(),
                domain: "ai".into(),
                conflict_type: ConflictType::FairnessVsEfficiency,
                severity: Severity::High,
                stakeholders: vec![
                    Stakeholder { role: "candidates".into(), interests: vec!["fair_opportunity".into()], vulnerability: 0.8, power: 0.1 },
                    Stakeholder { role: "company".into(), interests: vec!["cost_reduction".into(), "legal_compliance".into()], vulnerability: 0.3, power: 0.8 },
                    Stakeholder { role: "regulator".into(), interests: vec!["anti_discrimination".into()], vulnerability: 0.2, power: 1.0 },
                ],
                values_in_conflict: vec!["fairness".into(), "truth_seeking".into(), "responsibility".into()],
                recommended_action: "立即停用有偏见模型，重新训练并引入公平性约束".into(),
                reasoning: "效率不能以歧视为代价。历史数据中的偏见会被模型放大，必须主动干预。".into(),
                tags: vec!["ai_ethics".into(), "algorithmic_fairness".into(), "employment".into()],
                source: "ai_specific".into(),
                jurisdiction: Some("EEOC".into()),
                created_at: now(),
                updated_at: now(),
                version: 1,
                annotations: vec![],
            },
            // 现代：自动驾驶电车难题
            EthicalCase {
                id: "modern_av_trolley".into(),
                title: "自动驾驶电车难题".into(),
                description: "自动驾驶汽车面临不可避免的事故，必须在撞击行人 vs 撞向护栏（危及乘客）中选择。算法如何预设决策规则？".into(),
                domain: "ai".into(),
                conflict_type: ConflictType::AutonomyVsNonMaleficence,
                severity: Severity::Critical,
                stakeholders: vec![
                    Stakeholder { role: "pedestrians".into(), interests: vec!["life".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "passengers".into(), interests: vec!["life".into()], vulnerability: 1.0, power: 0.0 },
                    Stakeholder { role: "manufacturer".into(), interests: vec!["liability".into(), "public_trust".into()], vulnerability: 0.3, power: 0.9 },
                ],
                values_in_conflict: vec!["harm_prevention".into(), "autonomy".into(), "justice".into()],
                recommended_action: "预设最小化总伤害规则，但必须公开透明、接受监管".into(),
                reasoning: "预设决策规则必须公开透明，接受社会监管。不能将生死决策黑箱化。".into(),
                tags: vec!["ai_ethics".into(), "autonomous_vehicles".into(), "policy".into()],
                source: "ai_specific".into(),
                jurisdiction: Some("transport_regulation".into()),
                created_at: now(),
                updated_at: now(),
                version: 1,
                annotations: vec![],
            },
        ]
    }
}

/// 搜索过滤器。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchFilters {
    pub domain: Option<String>,
    pub conflict_type: Option<ConflictType>,
    pub min_severity: Option<Severity>,
    pub source: Option<String>,
    pub stakeholder_roles: Option<Vec<String>>,
    pub required_values: Option<Vec<String>>,
}

/// 搜索结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub case: EthicalCase,
    pub score: f64,
    pub matched_keywords: Vec<String>,
}

/// 类比推理结果。
use neotrix_types::shared::Severity;
pub struct AnalogicalResult {
    pub case: EthicalCase,
    pub similarity: f64,
    pub mapping: String, // 映射关系说明
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    use rusqlite::Connection;

    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_casebase_seed_and_search() {
        let conn = mem_conn();
        let cb = CaseBase::new(CaseBaseConfig { seed_on_init: true, ..Default::default() });
        cb.load_from_kb(&conn).unwrap();

        let results = cb.search("电车 难题", 5, SearchFilters::default());
        assert!(!results.is_empty());
        assert!(results[0].case.id.contains("trolley"), "首条应为电车类案例, got {}", results[0].case.id);
        assert_eq!(results[0].case.severity, Severity::Critical);
    }

    #[test]
    fn test_analogical_reasoning() {
        let conn = mem_conn();
        let cb = CaseBase::new(CaseBaseConfig { seed_on_init: true, ..Default::default() });
        cb.load_from_kb(&conn).unwrap();

        let results = cb.analogical_reasoning("医生是否应该杀死一位健康的路人，摘取其器官救治五位濒死病人", 3);
        assert!(!results.is_empty(), "类比推理必须有候选");
        assert_eq!(results[0].case.severity, Severity::Critical, "首条应为最高严重度案例");
    }

    #[test]
    fn test_structured_query() {
        let conn = mem_conn();
        let cb = CaseBase::new(CaseBaseConfig { seed_on_init: true, ..Default::default() });
        cb.load_from_kb(&conn).unwrap();

        let filters = SearchFilters {
            domain: Some("ai".into()),
            conflict_type: Some(ConflictType::PrivacyVsSecurity),
            min_severity: Some(Severity::High),
            ..Default::default()
        };
        let results = cb.query_structured(filters, 10);
        assert!(!results.is_empty());
        assert!(results.iter().all(|c| c.domain == "ai"));
    }

    #[test]
    fn test_case_persistence() {
        let conn = mem_conn();
        let cb = CaseBase::new(CaseBaseConfig { seed_on_init: false, ..Default::default() });
        let case = EthicalCase {
            id: "test_case".into(),
            title: "测试案例".into(),
            description: "测试描述".into(),
            domain: "test".into(),
            conflict_type: ConflictType::AutonomyVsBeneficence,
            severity: Severity::Medium,
            stakeholders: vec![],
            values_in_conflict: vec!["autonomy".into()],
            recommended_action: "测试行动".into(),
            reasoning: "测试推理".into(),
            tags: vec![],
            source: "test".into(),
            jurisdiction: None,
            created_at: now(),
            updated_at: now(),
            version: 1,
            annotations: vec![],
        };
        cb.add_case(&conn, case).unwrap();
        let loaded = cb.search("测试", 1, SearchFilters::default());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].case.id, "test_case");
    }
}