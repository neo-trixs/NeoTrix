//! Module Integration Patterns — 模块集成模式库
//!
//! 吸收 KB 经验:
//! - R-P42 (强化现有节点, 禁止平行适配器)
//! - 先探索接口再添加委托
//! - 优先扩展现有模块
//! - 单元测试验证
//! - 编译检查门禁

use serde::{Deserialize, Serialize};

/// 集成模式库
pub struct IntegrationPatternLibrary {
    patterns: Vec<IntegrationPattern>,
    active_integrations: Vec<ActiveIntegration>,
    config: IntegrationConfig,
}

/// 集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub max_integrations: usize,
    pub require_tests: bool,
    pub require_docs: bool,
    pub enable_pattern_suggestions: bool,
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            max_integrations: 50,
            require_tests: true,
            require_docs: true,
            enable_pattern_suggestions: true,
        }
    }
}

/// 集成模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPattern {
    pub pattern_id: String,
    pub name: String,
    pub description: String,
    pub category: PatternCategory,
    pub steps: Vec<String>,
    pub rules: Vec<String>,
    pub examples: Vec<String>,
}

/// 模式分类
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PatternCategory {
    Delegation,      // 委托模式
    Extension,       // 扩展模式
    Composition,     // 组合模式
    Adaptation,      // 适配模式
    Monitoring,      // 监控模式
}

/// 活跃集成
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveIntegration {
    pub integration_id: String,
    pub pattern_id: String,
    pub source_module: String,
    pub target_module: String,
    pub status: IntegrationStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// 集成状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationStatus {
    Planning,
    Implementing,
    Testing,
    Complete,
    Failed,
}

/// 集成计划
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPlan {
    pub plan_id: String,
    pub pattern_id: String,
    pub source_module: String,
    pub target_module: String,
    pub steps: Vec<PlanStep>,
    pub estimated_duration: u64,
}

/// 计划步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step_id: String,
    pub description: String,
    pub required: bool,
    pub dependencies: Vec<String>,
}

/// 集成检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationCheck {
    pub passed: bool,
    pub violations: Vec<RuleViolation>,
    pub suggestions: Vec<String>,
}

/// 规则违反
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleViolation {
    pub rule_id: String,
    pub rule_name: String,
    pub message: String,
    pub severity: ViolationSeverity,
}

/// 违反严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

impl IntegrationPatternLibrary {
    /// 创建新的集成模式库
    pub fn new() -> Self {
        let mut lib = Self {
            patterns: Vec::new(),
            active_integrations: Vec::new(),
            config: IntegrationConfig::default(),
        };
        lib.register_default_patterns();
        lib
    }

    /// 注册默认模式
    fn register_default_patterns(&mut self) {
        // 模式1: 委托模式 (R-P42)
        self.patterns.push(IntegrationPattern {
            pattern_id: "delegation_r42".into(),
            name: "委托模式 (R-P42 强化节点)".into(),
            description: "为现有模块添加功能委托，而非创建平行适配器".into(),
            category: PatternCategory::Delegation,
            steps: vec![
                "探索目标模块的公开接口 (trait/方法)".into(),
                "设计委托方法签名".into(),
                "实现委托方法".into(),
                "编写单元测试验证委托正确性".into(),
                "更新文档".into(),
            ],
            rules: vec![
                "R-P42: 强化现有节点, 禁止创建平行适配器模块".into(),
                "委托方法必须保持原有行为".into(),
                "必须有单元测试覆盖".into(),
            ],
            examples: vec![
                "为 nt_core 添加知识库查询委托".into(),
                "为 nt_mind 添加经验查询委托".into(),
            ],
        });

        // 模式2: 扩展模式
        self.patterns.push(IntegrationPattern {
            pattern_id: "extension".into(),
            name: "扩展模式".into(),
            description: "在现有模块内添加新的实现".into(),
            category: PatternCategory::Extension,
            steps: vec![
                "确认目标模块是否支持扩展".into(),
                "在模块内添加新实现".into(),
                "更新模块导出".into(),
                "编写单元测试".into(),
            ],
            rules: vec![
                "必须保持向后兼容".into(),
                "新功能应该通过现有接口暴露".into(),
            ],
            examples: vec![
                "为 nt_act 添加新的工具类型".into(),
            ],
        });

        // 模式3: 组合模式
        self.patterns.push(IntegrationPattern {
            pattern_id: "composition".into(),
            name: "组合模式".into(),
            description: "组合多个模块创建复合功能".into(),
            category: PatternCategory::Composition,
            steps: vec![
                "定义组合接口".into(),
                "创建组合结构体".into(),
                "实现模块间协调".into(),
                "编写集成测试".into(),
            ],
            rules: vec![
                "组合应该通过 trait 定义".into(),
                "避免模块间直接依赖".into(),
            ],
            examples: vec![
                "组合 nt_world + nt_memory 创建 RAG 管线".into(),
            ],
        });
    }

    /// 推荐集成模式
    pub fn recommend_pattern(&self, source: &str, target: &str) -> Option<&IntegrationPattern> {
        // 根据模块类型推荐模式
        if source.starts_with("nt_") && target.starts_with("nt_") {
            // 同域模块: 委托模式
            self.patterns.iter().find(|p| p.pattern_id == "delegation_r42")
        } else {
            // 跨域模块: 组合模式
            self.patterns.iter().find(|p| p.pattern_id == "composition")
        }
    }

    /// 创建集成计划
    pub fn create_plan(&self, source: &str, target: &str, pattern_id: &str) -> Option<IntegrationPlan> {
        let pattern = self.patterns.iter().find(|p| p.pattern_id == pattern_id)?;

        let steps: Vec<PlanStep> = pattern.steps.iter().enumerate().map(|(i, step)| {
            PlanStep {
                step_id: format!("step_{}", i),
                description: step.clone(),
                required: true,
                dependencies: if i > 0 { vec![format!("step_{}", i - 1)] } else { vec![] },
            }
        }).collect();

        let step_count = steps.len() as u64;
        Some(IntegrationPlan {
            plan_id: uuid::Uuid::new_v4().to_string(),
            pattern_id: pattern_id.to_string(),
            source_module: source.to_string(),
            target_module: target.to_string(),
            steps,
            estimated_duration: step_count * 1800, // 每步30分钟
        })
    }

    /// 检查集成是否符合规则
    pub fn check_integration(&self, source: &str, target: &str, pattern_id: &str) -> IntegrationCheck {
        let mut violations = Vec::new();

        // 检查是否创建平行适配器
        if pattern_id != "delegation_r42" && pattern_id != "extension" {
            violations.push(RuleViolation {
                rule_id: "R-P42".into(),
                rule_name: "强化现有节点".into(),
                message: "推荐使用委托模式或扩展模式, 避免创建平行适配器".into(),
                severity: ViolationSeverity::Medium,
            });
        }

        // 检查跨域依赖
        if source.starts_with("nt_core") && target.starts_with("nt_io") {
            violations.push(RuleViolation {
                rule_id: "LAYER_DEP".into(),
                rule_name: "层级依赖".into(),
                message: "L5 不应直接依赖 L1, 应通过 trait 抽象".into(),
                severity: ViolationSeverity::High,
            });
        }

        IntegrationCheck {
            passed: violations.is_empty(),
            violations,
            suggestions: vec![
                "优先使用委托模式扩展功能".into(),
                "避免创建平行适配器模块".into(),
            ],
        }
    }

    /// 获取所有模式
    pub fn patterns(&self) -> &[IntegrationPattern] {
        &self.patterns
    }

    /// 获取活跃集成
    pub fn active_integrations(&self) -> &[ActiveIntegration] {
        &self.active_integrations
    }
}
