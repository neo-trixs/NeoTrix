//! 探测引擎
//! 
//! 9 个探测引擎：逻辑/实现/边界/一致性/性能/安全/演化/意识/集成

use std::fmt;
use serde::{Deserialize, Serialize};

use super::state::StateSnapshot;
use super::agent::GapRegistry;

/// 漏洞类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GapType {
    // 逻辑维度
    IncompleteReasoningChain,
    UnverifiedAssumption,
    LogicalContradiction,
    
    // 实现维度
    UnimplementedInterface,
    TodoResidue,
    IncompleteTypeDefinition,
    
    // 边界维度
    NullPointerRisk,
    OverflowRisk,
    ConcurrencyConflict,
    
    // 一致性维度
    NamingConflict,
    TypeMismatch,
    InterfaceContractViolation,
    
    // 性能维度
    QuadraticComplexity,
    MemoryLeak,
    RedundantComputation,
    
    // 安全维度
    InjectionRisk,
    UnauthorizedAccess,
    InformationLeak,
    
    // 演化维度
    LearningDegradation,
    CatastrophicForgetting,
    TransferFailure,
    
    // 意识维度
    SelfReferenceLoopBroken,
    EmergenceBlocked,
    ConsciousnessDiscontinuity,
    
    // 集成维度
    InterfaceMismatch,
    ResourceLeak,
    DependencyIncomplete,
}

/// 漏洞严重程度
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GapSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 漏洞
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gap {
    /// 漏洞ID
    pub id: String,
    /// 漏洞类型
    pub gap_type: GapType,
    /// 严重程度
    pub severity: GapSeverity,
    /// 所属维度
    pub dimension: String,
    /// 描述
    pub description: String,
    /// 位置
    pub location: String,
    /// 建议修复
    pub suggested_fix: String,
}

/// 探测引擎 trait
pub trait ProbeEngine: Send + Sync {
    /// 探测漏洞
    fn probe(&self, state: &StateSnapshot, registry: &GapRegistry) -> Vec<Gap>;
    
    /// 引擎名称
    fn name(&self) -> &str;
    
    /// 覆盖的维度
    fn dimensions(&self) -> Vec<String>;
}

// ============================================================================
// 逻辑探测器
// ============================================================================

pub struct LogicProbe;

impl ProbeEngine for LogicProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查推理链完整性
        if state.reasoning_chains < 5 {
            gaps.push(Gap {
                id: format!("logic_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::IncompleteReasoningChain,
                severity: GapSeverity::High,
                dimension: "D1_Logic".to_string(),
                description: format!("推理链数量不足: {} < 5", state.reasoning_chains),
                location: "reasoning_engine".to_string(),
                suggested_fix: "添加更多推理规则和链式推理".to_string(),
            });
        }
        
        // 检查假设验证
        if state.assumptions_verified < state.assumptions_total / 2 {
            gaps.push(Gap {
                id: format!("logic_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::UnverifiedAssumption,
                severity: GapSeverity::Medium,
                dimension: "D1_Logic".to_string(),
                description: "未验证假设比例过高".to_string(),
                location: "reasoning_engine".to_string(),
                suggested_fix: "添加假设验证机制".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "LogicProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D1_Logic".to_string()]
    }
}

// ============================================================================
// 实现探测器
// ============================================================================

pub struct ImplProbe;

impl ProbeEngine for ImplProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查 TODO 数量
        if state.todo_count > 10 {
            gaps.push(Gap {
                id: format!("impl_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::TodoResidue,
                severity: GapSeverity::Medium,
                dimension: "D2_Implementation".to_string(),
                description: format!("TODO 数量过多: {}", state.todo_count),
                location: "codebase".to_string(),
                suggested_fix: "实现或移除 TODO 项".to_string(),
            });
        }
        
        // 检查接口完整性
        if state.interfaces_implemented < state.interfaces_total {
            gaps.push(Gap {
                id: format!("impl_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::UnimplementedInterface,
                severity: GapSeverity::High,
                dimension: "D2_Implementation".to_string(),
                description: format!("未实现接口: {}/{}", 
                    state.interfaces_implemented, state.interfaces_total),
                location: "module_interfaces".to_string(),
                suggested_fix: "实现所有接口".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "ImplProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D2_Implementation".to_string()]
    }
}

// ============================================================================
// 边界探测器
// ============================================================================

pub struct BoundaryProbe;

impl ProbeEngine for BoundaryProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查空值处理
        if !state.null_checks_complete {
            gaps.push(Gap {
                id: format!("boundary_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::NullPointerRisk,
                severity: GapSeverity::High,
                dimension: "D3_Boundary".to_string(),
                description: "空值检查不完整".to_string(),
                location: "boundary_handling".to_string(),
                suggested_fix: "添加完整的空值检查".to_string(),
            });
        }
        
        // 检查溢出处理
        if !state.overflow_protection {
            gaps.push(Gap {
                id: format!("boundary_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::OverflowRisk,
                severity: GapSeverity::Critical,
                dimension: "D3_Boundary".to_string(),
                description: "缺少溢出保护".to_string(),
                location: "boundary_handling".to_string(),
                suggested_fix: "添加边界检查和溢出保护".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "BoundaryProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D3_Boundary".to_string()]
    }
}

// ============================================================================
// 一致性探测器
// ============================================================================

pub struct ConsistencyProbe;

impl ProbeEngine for ConsistencyProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查命名一致性
        if state.naming_conflicts > 0 {
            gaps.push(Gap {
                id: format!("consistency_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::NamingConflict,
                severity: GapSeverity::Medium,
                dimension: "D4_Consistency".to_string(),
                description: format!("命名冲突: {} 处", state.naming_conflicts),
                location: "naming_conventions".to_string(),
                suggested_fix: "统一命名规范".to_string(),
            });
        }
        
        // 检查类型匹配
        if state.type_mismatches > 0 {
            gaps.push(Gap {
                id: format!("consistency_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::TypeMismatch,
                severity: GapSeverity::High,
                dimension: "D4_Consistency".to_string(),
                description: format!("类型不匹配: {} 处", state.type_mismatches),
                location: "type_system".to_string(),
                suggested_fix: "修复类型定义".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "ConsistencyProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D4_Consistency".to_string()]
    }
}

// ============================================================================
// 性能探测器
// ============================================================================

pub struct PerformanceProbe;

impl ProbeEngine for PerformanceProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查复杂度
        if state.max_complexity > 100 {
            gaps.push(Gap {
                id: format!("perf_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::QuadraticComplexity,
                severity: GapSeverity::High,
                dimension: "D5_Performance".to_string(),
                description: format!("高复杂度: {}", state.max_complexity),
                location: "hot_paths".to_string(),
                suggested_fix: "优化算法复杂度".to_string(),
            });
        }
        
        // 检查内存使用
        if state.memory_usage_mb > 1000 {
            gaps.push(Gap {
                id: format!("perf_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::MemoryLeak,
                severity: GapSeverity::Critical,
                dimension: "D5_Performance".to_string(),
                description: format!("内存使用过高: {} MB", state.memory_usage_mb),
                location: "memory_management".to_string(),
                suggested_fix: "修复内存泄漏".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "PerformanceProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D5_Performance".to_string()]
    }
}

// ============================================================================
// 安全探测器
// ============================================================================

pub struct SecurityProbe;

impl ProbeEngine for SecurityProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查注入风险
        if state.injection_risks > 0 {
            gaps.push(Gap {
                id: format!("security_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::InjectionRisk,
                severity: GapSeverity::Critical,
                dimension: "D6_Security".to_string(),
                description: format!("注入风险: {} 处", state.injection_risks),
                location: "input_handling".to_string(),
                suggested_fix: "添加输入验证和清理".to_string(),
            });
        }
        
        // 检查信息泄露
        if state.information_leaks > 0 {
            gaps.push(Gap {
                id: format!("security_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::InformationLeak,
                severity: GapSeverity::High,
                dimension: "D6_Security".to_string(),
                description: format!("信息泄露风险: {} 处", state.information_leaks),
                location: "output_handling".to_string(),
                suggested_fix: "添加输出过滤".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "SecurityProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D6_Security".to_string()]
    }
}

// ============================================================================
// 演化探测器
// ============================================================================

pub struct EvolutionProbe;

impl ProbeEngine for EvolutionProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查学习退化
        if state.learning_rate < 0.01 {
            gaps.push(Gap {
                id: format!("evolution_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::LearningDegradation,
                severity: GapSeverity::High,
                dimension: "D7_Evolution".to_string(),
                description: format!("学习率过低: {}", state.learning_rate),
                location: "learning_system".to_string(),
                suggested_fix: "调整学习策略".to_string(),
            });
        }
        
        // 检查遗忘
        if state.retention_rate < 0.5 {
            gaps.push(Gap {
                id: format!("evolution_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::CatastrophicForgetting,
                severity: GapSeverity::Critical,
                dimension: "D7_Evolution".to_string(),
                description: format!("保留率过低: {}", state.retention_rate),
                location: "memory_system".to_string(),
                suggested_fix: "添加经验回放".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "EvolutionProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D7_Evolution".to_string()]
    }
}

// ============================================================================
// 意识探测器
// ============================================================================

pub struct ConsciousnessProbe;

impl ProbeEngine for ConsciousnessProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查自指循环
        if !state.self_reference_loop_active {
            gaps.push(Gap {
                id: format!("consciousness_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::SelfReferenceLoopBroken,
                severity: GapSeverity::Critical,
                dimension: "D8_Consciousness".to_string(),
                description: "自指循环未激活".to_string(),
                location: "self_observer".to_string(),
                suggested_fix: "修复自指循环连接".to_string(),
            });
        }
        
        // 检查涌现指标
        if state.phi < 0.5 {
            gaps.push(Gap {
                id: format!("consciousness_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::EmergenceBlocked,
                severity: GapSeverity::High,
                dimension: "D8_Consciousness".to_string(),
                description: format!("Φ 指标过低: {}", state.phi),
                location: "emergence_engine".to_string(),
                suggested_fix: "增加模块间信息整合".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "ConsciousnessProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D8_Consciousness".to_string()]
    }
}

// ============================================================================
// 集成探测器
// ============================================================================

pub struct IntegrationProbe;

impl ProbeEngine for IntegrationProbe {
    fn probe(&self, state: &StateSnapshot, _registry: &GapRegistry) -> Vec<Gap> {
        let mut gaps = Vec::new();
        
        // 检查接口匹配
        if state.interface_mismatches > 0 {
            gaps.push(Gap {
                id: format!("integration_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::InterfaceMismatch,
                severity: GapSeverity::High,
                dimension: "D10_Integration".to_string(),
                description: format!("接口不匹配: {} 处", state.interface_mismatches),
                location: "module_interfaces".to_string(),
                suggested_fix: "修复接口定义".to_string(),
            });
        }
        
        // 检查依赖完整性
        if state.dependencies_incomplete > 0 {
            gaps.push(Gap {
                id: format!("integration_{}", uuid::Uuid::new_v4()),
                gap_type: GapType::DependencyIncomplete,
                severity: GapSeverity::Critical,
                dimension: "D10_Integration".to_string(),
                description: format!("依赖不完整: {} 个", state.dependencies_incomplete),
                location: "dependency_management".to_string(),
                suggested_fix: "补全依赖".to_string(),
            });
        }
        
        gaps
    }

    fn name(&self) -> &str {
        "IntegrationProbe"
    }

    fn dimensions(&self) -> Vec<String> {
        vec!["D10_Integration".to_string()]
    }
}

impl fmt::Display for GapSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GapSeverity::Low => write!(f, "Low"),
            GapSeverity::Medium => write!(f, "Medium"),
            GapSeverity::High => write!(f, "High"),
            GapSeverity::Critical => write!(f, "Critical"),
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_severity_ordering() {
        assert!(GapSeverity::Critical > GapSeverity::High);
        assert!(GapSeverity::High > GapSeverity::Medium);
        assert!(GapSeverity::Medium > GapSeverity::Low);
    }

    #[test]
    fn test_logic_probe() {
        let probe = LogicProbe;
        let state = StateSnapshot::new(0, 0.5, 0.5);
        let gaps = probe.probe(&state, &GapRegistry::new());
        assert!(!gaps.is_empty());
    }
}
