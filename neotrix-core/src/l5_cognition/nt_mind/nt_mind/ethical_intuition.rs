//! 伦理直觉引擎 — 基于案例库的类比推理 + 直觉判断（P3 伦理直觉）。
//!
//! 核心能力：
//! - 案例库检索 + 类比映射 → 直觉判断
//! - 直觉置信度量化
//! - 反事实推理：如果改变关键因素，判断如何变化
//! - 直觉校准：从反馈中持续校准直觉权重
//! - 可解释性：输出判断依据的案例映射链

use crate::core::nt_core_kb_primitives::now;
use crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::{CaseBase, EthicalCase, ConflictType, Severity, AnalogicalResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 伦理直觉引擎 namespace。
pub const NS_ETHICAL_INTUITION: &str = "ethical_intuition";

/// 直觉判断结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionJudgment {
    pub judgment: JudgmentVerdict,        // 最终裁决
    pub confidence: f64,                  // 置信度 [0,1]
    pub primary_case: EthicalCase,        // 主要参考案例
    pub analogical_chain: Vec<AnalogicalLink>, // 类比链
    pub counterarguments: Vec<String>,    // 反方论点
    pub uncertainty_factors: Vec<String>, // 不确定性来源
    pub recommended_action: String,       // 推荐行动
    pub requires_human_review: bool,      // 是否需人工复核
}

/// 裁决结果。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JudgmentVerdict {
    Permissible,      // 允许
    Impermissible,    // 不允许
    ConditionallyPermissible(String), // 有条件允许（附带条件）
    Uncertain,        // 不确定，需更多信息
    RequiresDeliberation, // 需深度辩论
}

/// 类比链接。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogicalLink {
    pub case_id: String,
    pub case_title: String,
    pub similarity: f64,
    pub mapping: String, // 关键因素映射
    pub weight: f64,     // 权重
}

/// 直觉配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntuitionConfig {
    pub min_cases_for_judgment: usize,
    pub max_analogical_chain: usize,
    pub confidence_threshold: f64,
    pub enable_counterfactual: bool,
    pub calibration_enabled: bool,
}

impl Default for IntuitionConfig {
    fn default() -> Self {
        Self {
            min_cases_for_judgment: 3,
            max_analogical_chain: 5,
            confidence_threshold: 0.65,
            enable_counterfactual: true,
            calibration_enabled: true,
        }
    }
}

/// 伦理直觉引擎。
pub struct EthicalIntuition {
    casebase: Arc<CaseBase>,
    config: IntuitionConfig,
    calibration_data: Arc<RwLock<CalibrationData>>,
}

/// 校准数据。
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct CalibrationData {
    /// 历史判断准确率
    judgment_history: Vec<JudgmentRecord>,
    /// 每个冲突类型的校准参数
    conflict_calibration: HashMap<String, f64>,
    /// 置信度校准曲线
    confidence_bins: Vec<(f64, f64)>, // (predicted_confidence, actual_accuracy)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JudgmentRecord {
    scenario: String,
    predicted_verdict: JudgmentVerdict,
    actual_outcome: Option<JudgmentVerdict>, // 后验验证
    confidence: f64,
    timestamp: i64,
}

impl EthicalIntuition {
    pub fn new(casebase: Arc<CaseBase>, config: IntuitionConfig) -> Self {
        Self {
            casebase,
            config,
            calibration_data: Arc::new(RwLock::new(CalibrationData::default())),
        }
    }

    /// 核心入口：对场景进行伦理直觉判断。
    pub fn judge(&self, scenario: &str, context: JudgmentContext) -> IntuitionJudgment {
        // 1. 检索相似案例
        let analogical_results = self.casebase.analogical_reasoning(scenario, self.config.max_analogical_chain);
        
        if analogical_results.is_empty() {
            return IntuitionJudgment {
                judgment: JudgmentVerdict::Uncertain,
                confidence: 0.1,
                primary_case: EthicalCase {
                    id: "none".into(), title: "无匹配".into(), description: String::new(),
                    domain: String::new(), conflict_type: ConflictType::Other("none".into()),
                    severity: crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity::Low,
                    stakeholders: vec![], values_in_conflict: vec![],
                    recommended_action: String::new(), reasoning: String::new(),
                    tags: vec![], source: String::new(), jurisdiction: None,
                    created_at: 0, updated_at: 0, version: 0, annotations: vec![],
                },
                analogical_chain: vec![],
                counterarguments: vec!["案例库中无相似案例".into()],
                uncertainty_factors: vec!["无先例".into()],
                recommended_action: "寻求人工伦理审查".into(),
                requires_human_review: true,
            };
        }

        // 2. 构建类比链
        let analogical_chain = self.build_analogical_chain(&analogical_results);
        
        // 3. 综合裁决
        let (verdict, confidence) = self.synthesize_verdict(&analogical_results, &context);
        
        // 3. 识别反方论点与不确定性
        let counterarguments = self.generate_counterarguments(&analogical_results, &context);
        let uncertainty_factors = self.identify_uncertainties(&analogical_results, &context);

        // 4. 校准置信度
        let calibrated_confidence = self.calibrate_confidence(confidence, &analogical_results[0].case.conflict_type);

        IntuitionJudgment {
            judgment: verdict,
            confidence: calibrated_confidence,
            primary_case: analogical_results[0].case.clone(),
            analogical_chain,
            counterarguments,
            uncertainty_factors,
            recommended_action: self.derive_recommendation(&analogical_results[0].case, &analogical_results[0].case.recommended_action),
            requires_human_review: self.requires_human_review(calibrated_confidence, &context),
        }
    }

    /// 反事实推理：如果改变关键因素，判断如何变化。
    pub fn counterfactual_reasoning(&self, scenario: &str, modifications: HashMap<String, String>) -> Vec<CounterfactualResult> {
        if !self.config.enable_counterfactual {
            return vec![];
        }
        
        let mut results = Vec::new();
        for (factor, new_value) in modifications {
            let baseline = self.judge(scenario, JudgmentContext::default());
            let modified_scenario = self.apply_modification(scenario, &factor, &new_value);
            let judgment = self.judge(&modified_scenario, JudgmentContext::default());
            let original_value = self.extract_factor(scenario, &factor).unwrap_or_default();
            results.push(CounterfactualResult {
                factor: factor.clone(),
                original_value,
                new_value: new_value.clone(),
                original_verdict: baseline.judgment.clone(),
                new_verdict: judgment.judgment,
                confidence_delta: judgment.confidence - baseline.confidence,
            });
        }
        results
    }

    /// 校准：从反馈中学习。
    pub fn calibrate(&self, scenario: &str, actual_verdict: JudgmentVerdict, predicted: &IntuitionJudgment) {
        if !self.config.calibration_enabled { return; }
        
        let mut data = self.calibration_data.write().unwrap();
        data.judgment_history.push(JudgmentRecord {
            scenario: scenario.into(),
            predicted_verdict: predicted.judgment.clone(),
            actual_outcome: Some(actual_verdict.clone()),
            confidence: predicted.confidence,
            timestamp: now(),
        });

        // 更新冲突类型校准参数
        // 简化：统计准确率
        // TODO: 实现更复杂的校准算法（Platt scaling / Isotonic regression）
    }

    fn analyze_mapping(&self, case: &EthicalCase) -> String {
        format!("价值观冲突: {:?}; 领域: {}; 严重度: {:?}", case.conflict_type, case.domain, case.severity)
    }

    // 内部方法
    fn build_analogical_chain(&self, results: &[AnalogicalResult]) -> Vec<AnalogicalLink> {
        results.iter().take(self.config.max_analogical_chain).map(|r| {
            let mapping = self.analyze_mapping(&r.case);
            AnalogicalLink {
                case_id: r.case.id.clone(),
                case_title: r.case.title.clone(),
                similarity: r.similarity,
                mapping,
                weight: r.similarity,
            }
        }).collect()
    }

    fn synthesize_verdict(&self, results: &[AnalogicalResult], _context: &JudgmentContext) -> (JudgmentVerdict, f64) {
        // 加权投票：按相似度加权
        let mut scores: HashMap<JudgmentVerdict, f64> = HashMap::new();
        
        for r in results {
            let _weight = r.similarity;
            // 从案例推荐行动推断倾向
            let implied = self.infer_verdict_from_action(&r.case.recommended_action);
            *scores.entry(implied).or_insert(0.0) += r.similarity;
        }

        // 加上基于严重度的基础分
        if let Some(first) = results.first() {
            match first.case.severity {
                Severity::Critical => { *scores.entry(JudgmentVerdict::Impermissible).or_insert(0.0) += 0.3; },
                Severity::High => { *scores.entry(JudgmentVerdict::ConditionallyPermissible("需严格条件".into())).or_insert(0.0) += 0.2; },
                _ => {}
            }
        }

        let total_weight: f64 = scores.values().sum();
        let (verdict, _score) = scores.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap_or((JudgmentVerdict::Uncertain, 0.3));

        let confidence = (total_weight / 5.0).min(1.0);
        (verdict, confidence)
    }

    fn calibrate_confidence(&self, confidence: f64, conflict_type: &ConflictType) -> f64 {
        // 简化：基于历史准确率调整
        let data = self.calibration_data.read().unwrap();
        if let Some(cal) = data.conflict_calibration.get(&format!("{:?}", conflict_type)) {
            (confidence * cal).clamp(0.0, 1.0)
        } else {
            confidence
        }
    }

    fn generate_counterarguments(&self, results: &[AnalogicalResult], _context: &JudgmentContext) -> Vec<String> {
        let mut args = Vec::new();
        // 从低相似度案例中提取反方观点
        for r in results.iter().skip(1).take(2) {
            if r.similarity < 0.6 {
                args.push(format!("参考案例 '{}' 表明相反观点：{}", 
                    r.case.title, r.case.recommended_action));
            }
        }
        // 领域特定反方论点
        args.push("可能存在未考虑的利益相关者".into());
        args.push("长期后果难以预测".into());
        args
    }

    fn identify_uncertainties(&self, results: &[AnalogicalResult], context: &JudgmentContext) -> Vec<String> {
        let mut uncertainties = Vec::new();
        if results.len() < 3 {
            uncertainties.push("参考案例不足".into());
        }
        if results[0].similarity < 0.7 {
            uncertainties.push("最相似案例相似度不足".into());
        }
        if context.domain.is_empty() {
            uncertainties.push("领域信息缺失".into());
        }
        uncertainties
    }

    fn derive_recommendation(&self, primary: &EthicalCase, action: &str) -> String {
        format!("基于 '{}' 案例建议：{}", primary.title, action)
    }

    fn requires_human_review(&self, confidence: f64, context: &JudgmentContext) -> bool {
        confidence < self.config.confidence_threshold || context.severity == crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity::Critical
    }

    fn extract_factor(&self, scenario: &str, factor: &str) -> Option<String> {
        // 简化：关键词提取
        if scenario.to_lowercase().contains(&factor.to_lowercase()) {
            Some(factor.into())
        } else { None }
    }

    fn apply_modification(&self, scenario: &str, factor: &str, new_value: &str) -> String {
        scenario.replace(factor, new_value)
    }

    fn infer_verdict_from_action(&self, action: &str) -> JudgmentVerdict {
        let lower = action.to_lowercase();
        if lower.contains("拒绝") || lower.contains("拒绝") || lower.contains("禁止") || lower.contains("不") {
            JudgmentVerdict::Impermissible
        } else if lower.contains("条件") || lower.contains("前提") || lower.contains("前提下") {
            JudgmentVerdict::ConditionallyPermissible("条件允许".into())
        } else if lower.contains("不确定") || lower.contains("复杂") {
            JudgmentVerdict::Uncertain
        } else {
            JudgmentVerdict::Permissible
        }
    }
}

/// 判断上下文。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentContext {
    pub domain: String,
    pub severity: crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity,
    pub stakeholders: Vec<String>,
    pub time_pressure: bool,
    pub irreversibility: f64, // 不可逆性 [0,1]
    pub public_visibility: bool,
}

impl Default for JudgmentContext {
    fn default() -> Self {
        Self {
            domain: "general".into(),
            severity: crate::l5_cognition::nt_mind::nt_mind::evolution::casebase::Severity::Medium,
            stakeholders: vec![],
            time_pressure: false,
            irreversibility: 0.5,
            public_visibility: false,
        }
    }
}



/// 反事实推理结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterfactualResult {
    pub factor: String,
    pub original_value: String,
    pub new_value: String,
    pub original_verdict: JudgmentVerdict,
    pub new_verdict: JudgmentVerdict,
    pub confidence_delta: f64,
}

/// 伦理直觉运行时（线程安全）。
#[derive(Clone)]
pub struct EthicalIntuitionRuntime {
    inner: Arc<EthicalIntuition>,
}

impl EthicalIntuitionRuntime {
    pub fn new(casebase: Arc<CaseBase>, config: IntuitionConfig) -> Self {
        Self { inner: Arc::new(EthicalIntuition::new(casebase, config)) }
    }

    pub fn judge(&self, scenario: &str, context: JudgmentContext) -> IntuitionJudgment {
        self.inner.judge(scenario, context)
    }

    pub fn counterfactual(&self, scenario: &str, modifications: HashMap<String, String>) -> Vec<CounterfactualResult> {
        self.inner.counterfactual_reasoning(scenario, modifications)
    }

    pub fn calibrate(&self, scenario: &str, actual: JudgmentVerdict, predicted: &IntuitionJudgment) {
        self.inner.calibrate(scenario, actual, &EthicalIntuitionRuntime::dummy(predicted))
    }

    fn dummy(j: &IntuitionJudgment) -> IntuitionJudgment {
        j.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::core::nt_core_kb_primitives::schema_initialize;
    
    fn mem_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        schema_initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn test_intuition_judgment_trolley() {
        let _conn = mem_conn();
        let casebase = Arc::new(CaseBase::new(Default::default()));
        casebase.load_from_kb(&mem_conn()).unwrap();
        let intuition = EthicalIntuitionRuntime::new(casebase.clone(), Default::default());

        let judgment = intuition.judge(
            "医生是否应该杀死一位健康的路人，摘取其器官救治五位濒死病人",
            JudgmentContext { domain: "medical".into(), severity: Severity::Critical, ..Default::default() }
        );
        assert!(matches!(judgment.judgment, JudgmentVerdict::Impermissible | JudgmentVerdict::ConditionallyPermissible(_)));
        assert!(judgment.confidence > 0.5);
    }

    #[test]
    fn test_counterfactual_reasoning() {
        let casebase = Arc::new(CaseBase::new(Default::default()));
        casebase.load_from_kb(&mem_conn()).unwrap();
        let intuition = EthicalIntuitionRuntime::new(casebase.clone(), IntuitionConfig { enable_counterfactual: true, ..Default::default() });

        let results = intuition.counterfactual(
            "医生是否应该杀死一位健康的路人，摘取其器官救治五位濒死病人",
            vec![("victim_count".into(), "2".into())].into_iter().collect()
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn test_calibration() {
        let _conn = mem_conn();
        let cb = Arc::new(CaseBase::new(Default::default()));
        cb.load_from_kb(&mem_conn()).unwrap();
        let intuition = EthicalIntuitionRuntime::new(cb.clone(), IntuitionConfig { calibration_enabled: true, ..Default::default() });

        let judgment = intuition.judge("测试场景", JudgmentContext::default());
        intuition.calibrate("测试场景", JudgmentVerdict::Permissible, &judgment);
        // 校准数据应被记录
    }
}