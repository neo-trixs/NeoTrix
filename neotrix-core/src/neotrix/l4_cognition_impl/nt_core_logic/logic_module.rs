//! nt_core_logic — 核心逻辑系统
//!
//! 符号逻辑处理和真值代数
//! 节点: nt_core_logic (L4)
//! Provides: symbolic_logic, truth_algebra
//! Requires: nt_core_traits, serde
//! Rune: Crimson, Alabaster

#![forbid(unsafe_code)]

use crate::core::nt_core_traits::{CapabilityNode, RuneSocket, SelfTest};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogicConfig {
    /// 是否启用三值逻辑
    pub three_valued: bool,
    /// 未知值
    pub unknown: f32,
}

impl Default for LogicConfig {
    fn default() -> Self {
        Self {
            three_valued: false,
            unknown: 0.5,
        }
    }
}

/// 逻辑运算符
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LogicOperator {
    /// 合取
    And,
    /// 合取
    Or,
    /// 非
    Not,
    /// 蕴涵
    Implies,
    /// 等价
    Iff,
}

/// 真值
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TruthValue {
    pub value: f32,
}

impl TruthValue {
    pub const TRUE: TruthValue = TruthValue { value: 1.0 };
    pub const FALSE: TruthValue = TruthValue { value: 0.0 };
    pub const UNKNOWN: TruthValue = TruthValue { value: 0.5 };

    /// 是否为模糊真值 (介于 0/1 之间) — 用于三值逻辑判定
    fn is_fuzzy(&self) -> bool {
        self.value != 0.0 && self.value != 1.0
    }

    pub fn and(self, other: TruthValue) -> TruthValue {
        if self.is_fuzzy() || other.is_fuzzy() {
            TruthValue {
                value: self.value * other.value,
            }
        } else {
            TruthValue {
                value: if self.value == 1.0 && other.value == 1.0 {
                    1.0
                } else {
                    0.0
                },
            }
        }
    }

    pub fn or(self, other: TruthValue) -> TruthValue {
        if self.is_fuzzy() || other.is_fuzzy() {
            TruthValue {
                value: self.value + other.value - self.value * other.value,
            }
        } else {
            TruthValue {
                value: if self.value == 0.0 && other.value == 0.0 {
                    0.0
                } else {
                    1.0
                },
            }
        }
    }

    pub fn not(self) -> TruthValue {
        if self.is_fuzzy() {
            TruthValue {
                value: 1.0 - self.value,
            }
        } else {
            TruthValue {
                value: if self.value == 0.0 { 1.0 } else { 0.0 },
            }
        }
    }

    pub fn implies(self, other: TruthValue) -> TruthValue {
        // material implication: ¬A ∨ B
        if self.is_fuzzy() || other.is_fuzzy() {
            TruthValue {
                value: 1.0 - self.value + self.value * other.value,
            }
        } else {
            TruthValue {
                value: if self.value == 1.0 && other.value == 0.0 {
                    0.0
                } else {
                    1.0
                },
            }
        }
    }

    pub fn iff(self, other: TruthValue) -> TruthValue {
        // equivalence: (A → B) ∧ (B → A)
        let imp1 = self.clone().implies(other.clone());
        let imp2 = other.implies(self);
        imp1.and(imp2)
    }
}

impl CapabilityNode for TruthValue {
    fn node_id(&self) -> &str {
        "nt_core_logic"
    }
    fn provides(&self) -> Vec<String> {
        vec!["symbolic_logic".into(), "truth_algebra".into()]
    }
    fn requires(&self) -> Vec<String> {
        vec!["nt_core_traits".into(), "serde".into()]
    }
    fn rune_sockets(&self) -> Vec<RuneSocket> {
        vec![RuneSocket::Crimson, RuneSocket::Alabaster]
    }
    fn constellation_level(&self) -> u8 {
        1
    }
    fn promote_constellation(&mut self) -> bool {
        true
    }
}

impl SelfTest for TruthValue {
    fn self_test(&self) -> Result<(), Vec<String>> {
        let inner = (|| -> Result<(), crate::core::nt_core_error::NeoTrixError> {
            let tv = TruthValue::TRUE;
            let result = tv.clone().and(TruthValue::FALSE);
            assert_eq!(result, TruthValue::FALSE);

            let result2 = tv.not();
            assert_eq!(result2, TruthValue::FALSE);

            Ok(())
        })();
        inner.map_err(|e| vec![e.to_string()])
    }
    fn name(&self) -> &str {
        "nt_core_logic_truthvalue"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_truthvalue_self_test() {
        let tv = TruthValue::TRUE;
        assert!(tv.self_test().is_ok());
    }
}
