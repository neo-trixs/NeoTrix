//! Trade Supplier Management — 供应商评估与管理模块
//!
//! 对标 TMS 平台的供应商管理能力：
//! - 供应商档案 (资质/认证/产能)
//! - 供应商评估 (质量/交期/价格/服务 四维评分)
//! - 供应商分级 (A/B/C/D)
//! - 供应商对比 (多维度打分)
//! - 合格供应商清单 (AVL)

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 供应商评估
// ============================================================

/// 评估维度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierEvaluation {
    /// 质量评分 (0-100)
    pub quality_score: f64,
    /// 交期评分 (0-100)
    pub delivery_score: f64,
    /// 价格竞争力 (0-100)
    pub price_score: f64,
    /// 服务响应 (0-100)
    pub service_score: f64,
    /// 综合评分 (加权)
    pub overall_score: f64,
    /// 评估日期
    pub evaluated_at: u64,
    /// 评估人
    pub evaluator: String,
    /// 备注
    pub notes: Option<String>,
}

impl SupplierEvaluation {
    /// 计算加权综合评分 (默认权重: 质量40%, 交期25%, 价格20%, 服务15%)
    pub fn calculate_overall(
        quality: f64,
        delivery: f64,
        price: f64,
        service: f64,
    ) -> f64 {
        quality * 0.40 + delivery * 0.25 + price * 0.20 + service * 0.15
    }
}

/// 供应商等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupplierGrade {
    /// A级: 优秀 (综合>=85)
    A,
    /// B级: 良好 (综合>=70)
    B,
    /// C级: 合格 (综合>=55)
    C,
    /// D级: 待改进 (综合<55)
    D,
    /// 未评估
    Unrated,
}

impl SupplierGrade {
    pub fn from_score(score: f64) -> Self {
        if score >= 85.0 { Self::A }
        else if score >= 70.0 { Self::B }
        else if score >= 55.0 { Self::C }
        else { Self::D }
    }
}

// ============================================================
// 2. 供应商档案
// ============================================================

/// 供应商档案 (扩展 data_model::Supplier)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierProfile {
    pub id: String,
    pub name: String,
    pub country: String,
    pub city: Option<String>,
    pub main_products: Vec<String>,
    pub certifications: Vec<String>,
    pub monthly_capacity: Option<u64>,
    pub lead_time_days: Option<u32>,
    pub min_order_qty: Option<u64>,
    pub payment_terms: Option<String>,
    pub contact_info: HashMap<String, String>,
    pub evaluation: Option<SupplierEvaluation>,
    pub grade: SupplierGrade,
    /// 合格供应商清单 (AVL) 标记
    pub on_avl: bool,
    /// 合作次数
    pub cooperation_count: u32,
    /// 累计采购金额 (USD)
    pub total_purchased: f64,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Default for SupplierProfile {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            country: String::new(),
            city: None,
            main_products: Vec::new(),
            certifications: Vec::new(),
            monthly_capacity: None,
            lead_time_days: None,
            min_order_qty: None,
            payment_terms: None,
            contact_info: HashMap::new(),
            evaluation: None,
            grade: SupplierGrade::Unrated,
            on_avl: false,
            cooperation_count: 0,
            total_purchased: 0.0,
            created_at: 0,
            updated_at: 0,
        }
    }
}

/// 供应商对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierComparison {
    pub suppliers: Vec<SupplierProfile>,
    pub best_quality: Option<String>,
    pub best_delivery: Option<String>,
    pub best_price: Option<String>,
    pub best_overall: Option<String>,
}

// ============================================================
// 3. 供应商管理引擎
// ============================================================

/// 供应商评估与管理引擎
pub struct SupplierMgmtEngine {
    suppliers: HashMap<String, SupplierProfile>,
}

impl Default for SupplierMgmtEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SupplierMgmtEngine {
    pub fn new() -> Self {
        Self {
            suppliers: HashMap::new(),
        }
    }

    /// 创建供应商
    pub fn create_supplier(&mut self, profile: SupplierProfile) -> SupplierProfile {
        let id = profile.id.clone();
        self.suppliers.insert(id, profile.clone());
        profile
    }

    /// 评估供应商
    pub fn evaluate_supplier(
        &mut self,
        supplier_id: &str,
        evaluation: SupplierEvaluation,
    ) -> Result<SupplierGrade, String> {
        let now = self.current_timestamp();
        let supplier = self
            .suppliers
            .get_mut(supplier_id)
            .ok_or_else(|| format!("Supplier {} not found", supplier_id))?;
        let grade = SupplierGrade::from_score(evaluation.overall_score);
        supplier.evaluation = Some(evaluation);
        supplier.grade = grade;
        supplier.updated_at = now;
        Ok(grade)
    }

    /// 加入/移除合格供应商清单
    pub fn set_avl_status(
        &mut self,
        supplier_id: &str,
        on_avl: bool,
    ) -> Result<(), String> {
        let now = self.current_timestamp();
        let supplier = self
            .suppliers
            .get_mut(supplier_id)
            .ok_or_else(|| format!("Supplier {} not found", supplier_id))?;
        supplier.on_avl = on_avl;
        supplier.updated_at = now;
        Ok(())
    }

    /// 多供应商对比
    pub fn compare_suppliers(
        &self,
        supplier_ids: &[String],
    ) -> SupplierComparison {
        let profiles: Vec<SupplierProfile> = supplier_ids
            .iter()
            .filter_map(|id| self.suppliers.get(id).cloned())
            .collect();
        let best_quality = profiles
            .iter()
            .filter_map(|s| s.evaluation.as_ref().map(|e| (s.id.clone(), e.quality_score)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
        let best_delivery = profiles
            .iter()
            .filter_map(|s| s.evaluation.as_ref().map(|e| (s.id.clone(), e.delivery_score)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
        let best_price = profiles
            .iter()
            .filter_map(|s| s.evaluation.as_ref().map(|e| (s.id.clone(), e.price_score)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
        let best_overall = profiles
            .iter()
            .filter_map(|s| s.evaluation.as_ref().map(|e| (s.id.clone(), e.overall_score)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id);
        SupplierComparison {
            suppliers: profiles,
            best_quality,
            best_delivery,
            best_price,
            best_overall,
        }
    }

    /// 获取合格供应商清单
    pub fn avl_list(&self) -> Vec<&SupplierProfile> {
        self.suppliers.values().filter(|s| s.on_avl).collect()
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================
// 4. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_supplier() {
        let mut engine = SupplierMgmtEngine::new();
        let profile = SupplierProfile {
            id: "SUP-001".into(),
            name: "Best Valve Co".into(),
            country: "CN".into(),
            ..Default::default()
        };
        engine.create_supplier(profile);
        let eval = SupplierEvaluation {
            quality_score: 90.0,
            delivery_score: 85.0,
            price_score: 75.0,
            service_score: 80.0,
            overall_score: SupplierEvaluation::calculate_overall(90.0, 85.0, 75.0, 80.0),
            evaluated_at: 0,
            evaluator: "admin".into(),
            notes: None,
        };
        let grade = engine.evaluate_supplier("SUP-001", eval).unwrap();
        assert_eq!(grade, SupplierGrade::A);
    }

    #[test]
    fn test_avl_list() {
        let mut engine = SupplierMgmtEngine::new();
        let mut p1 = SupplierProfile { id: "S1".into(), name: "A".into(), ..Default::default() };
        p1.on_avl = true;
        engine.create_supplier(p1);
        let p2 = SupplierProfile { id: "S2".into(), name: "B".into(), ..Default::default() };
        engine.create_supplier(p2);
        assert_eq!(engine.avl_list().len(), 1);
    }

    #[test]
    fn test_compare_suppliers() {
        let mut engine = SupplierMgmtEngine::new();
        engine.create_supplier(SupplierProfile { id: "S1".into(), name: "A".into(), ..Default::default() });
        engine.create_supplier(SupplierProfile { id: "S2".into(), name: "B".into(), ..Default::default() });
        engine.evaluate_supplier("S1", SupplierEvaluation {
            quality_score: 90.0, delivery_score: 80.0, price_score: 70.0, service_score: 85.0,
            overall_score: 82.0, evaluated_at: 0, evaluator: "".into(), notes: None,
        }).unwrap();
        engine.evaluate_supplier("S2", SupplierEvaluation {
            quality_score: 75.0, delivery_score: 90.0, price_score: 85.0, service_score: 70.0,
            overall_score: 80.0, evaluated_at: 0, evaluator: "".into(), notes: None,
        }).unwrap();
        let cmp = engine.compare_suppliers(&["S1".into(), "S2".into()]);
        assert_eq!(cmp.best_quality, Some("S1".into()));
        assert_eq!(cmp.best_price, Some("S2".into()));
    }
}
