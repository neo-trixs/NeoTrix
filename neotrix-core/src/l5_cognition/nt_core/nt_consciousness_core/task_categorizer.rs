//! 任务分类器 (TaskCategorizer)
//! 
//! 吸收自 NERV-BREAK-5.6 的消息分类机制
//! - 关键词匹配
//! - 任务路由
//! - 优先级排序

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 任务分类器
pub struct TaskCategorizer {
    /// 分类规则。**保序**（Vec 而非 HashMap）。
    ///
    /// 2026-09-28: 原为 `HashMap<String, Vec<String>>`, 而 Rust 的 HashMap
    /// **每个进程迭代顺序随机**。categorize 用 `score > best_score` 严格大于,
    /// 于是在**得分并列**时「先被迭代到的类别」获胜 —— 而并列在真实输入里很常见:
    /// 实测 "请帮我分析这段代码" 同时命中 reasoning 的「分析」与 coding 的
    /// 「代码」, 两者都 1/6 完全并列, 于是分类结果**每次运行都可能不同**
    /// (测试表现为 flaky, 约一半概率失败)。
    /// 改为 Vec 保序: 迭代确定, 并列时由**声明顺序**决出 —— 先声明的规则获胜,
    /// 这既确定, 也符合直觉(主规则优先)。全仓无外部消费者使用该字段。
    pub categories: Vec<(String, Vec<String>)>,
    /// 分类历史
    pub history: Vec<_ClassificationRecord>,
}

/// 分类记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _ClassificationRecord {
    /// 记录ID
    pub id: String,
    /// 输入
    pub input: String,
    /// 分类结果
    pub category: String,
    /// 置信度
    pub confidence: f64,
    /// 时间戳
    pub timestamp: String,
}

impl TaskCategorizer {
    /// 创建新的任务分类器
    pub fn new() -> Self {
        Self {
            categories: Vec::new(),
            history: Vec::new(),
        }
    }

    /// 添加分类规则
    pub(crate) fn _add_category(&mut self, name: String, keywords: Vec<String>) {
        // 同名覆盖时保持原位置, 不改变声明顺序
        match self.categories.iter_mut().find(|(n, _)| *n == name) {
            Some(slot) => slot.1 = keywords,
            None => self.categories.push((name, keywords)),
        }
    }

    /// 分类输入
    pub fn categorize(&mut self, input: &str) -> (String, f64) {
        let input_lower = input.to_lowercase();
        let mut best_category = "general".to_string();
        let mut best_score = 0.0;

        // 保序迭代 ⇒ 并列时先声明者胜, 结果确定 (见字段注释)
        for (category, keywords) in &self.categories {
            let matches = keywords.iter()
                .filter(|k| input_lower.contains(k.as_str()))
                .count();
            
            let score = matches as f64 / keywords.len() as f64;
            
            if score > best_score {
                best_score = score;
                best_category = category.clone();
            }
        }

        // 记录分类
        let record = _ClassificationRecord {
            id: format!("cls_{}", uuid::Uuid::new_v4()),
            input: input.to_string(),
            category: best_category.clone(),
            confidence: best_score,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.history.push(record);

        (best_category, best_score)
    }

    /// 批量分类
    pub(crate) fn _categorize_batch(&mut self, inputs: &[String]) -> Vec<(String, f64)> {
        inputs.iter().map(|input| self.categorize(input)).collect()
    }

    /// 获取分类统计
    pub fn stats(&self) -> _CategorizerStats {
        let mut category_counts = HashMap::new();
        for record in &self.history {
            *category_counts.entry(record.category.clone()).or_insert(0) += 1;
        }

        _CategorizerStats {
            total_categories: self.categories.len(),
            total_classifications: self.history.len(),
            category_distribution: category_counts,
        }
    }

    /// 创建默认分类器
    pub(crate) fn _default_categories() -> Self {
        let mut categorizer = Self::new();
        
        categorizer._add_category(
            "reasoning".to_string(),
            vec![
                "推理".to_string(),
                "reason".to_string(),
                "分析".to_string(),
                "analyze".to_string(),
                "为什么".to_string(),
                "why".to_string(),
            ],
        );
        
        categorizer._add_category(
            "coding".to_string(),
            vec![
                "代码".to_string(),
                "code".to_string(),
                "编程".to_string(),
                "program".to_string(),
                "实现".to_string(),
                "implement".to_string(),
            ],
        );
        
        categorizer._add_category(
            "research".to_string(),
            vec![
                "研究".to_string(),
                "research".to_string(),
                "搜索".to_string(),
                "search".to_string(),
                "查找".to_string(),
                "find".to_string(),
            ],
        );
        
        categorizer._add_category(
            "creative".to_string(),
            vec![
                "创作".to_string(),
                "create".to_string(),
                "写作".to_string(),
                "write".to_string(),
                "设计".to_string(),
                "design".to_string(),
            ],
        );
        
        categorizer
    }
}

/// 分类器统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _CategorizerStats {
    pub total_categories: usize,
    pub total_classifications: usize,
    pub category_distribution: HashMap<String, u32>,
}

impl std::fmt::Display for _CategorizerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        TaskCategorizer 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总分类数:        {}", self.total_categories)?;
        writeln!(f, "总分类次数:      {}", self.total_classifications)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "分类分布:")?;
        for (category, count) in &self.category_distribution {
            writeln!(f, "  {}: {}", category, count)?;
        }
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorizer() {
        let mut categorizer = TaskCategorizer::_default_categories();
        let (category, confidence) = categorizer.categorize("请帮我分析这段代码");
        
        assert_eq!(category, "reasoning");
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_general_category() {
        let mut categorizer = TaskCategorizer::_default_categories();
        let (category, _) = categorizer.categorize("今天天气真好");
        
        assert_eq!(category, "general");
    }
}
