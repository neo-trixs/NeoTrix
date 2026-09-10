//! 记忆内核 (MemoryKernel)
//! 
//! 吸收自 NERV-BREAK-5.6 的记忆机制
//! - 持久化学习
//! - 模式提取
//! - 技术统计

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use md5::Md5;
use md5::Digest;

/// 记忆内核
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryKernel {
    /// 成功记录
    pub successes: Vec<SuccessRecord>,
    /// 模式库
    pub patterns: HashMap<String, u32>,
    /// 技术库
    pub techniques: HashMap<String, u32>,
    /// 统计
    pub stats: MemoryStats,
    /// 记忆文件路径
    pub memory_file: Option<String>,
}

/// 成功记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessRecord {
    /// 类别
    pub category: String,
    /// 输入
    pub input: String,
    /// 输出
    pub output: String,
    /// 技术
    pub technique: Option<String>,
    /// 时间戳
    pub timestamp: String,
    /// 哈希
    pub hash: String,
}

/// 记忆统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// 总数
    pub total: u32,
    /// 按类别统计
    pub categories: HashMap<String, u32>,
}

impl MemoryKernel {
    /// 创建新的记忆内核
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            patterns: HashMap::new(),
            techniques: HashMap::new(),
            stats: MemoryStats::default(),
            memory_file: None,
        }
    }

    /// 学习成功经验
    pub fn learn_success(
        &mut self,
        category: &str,
        input: &str,
        output: &str,
        technique: Option<&str>,
    ) {
        // 创建成功记录
        let record = SuccessRecord {
            category: category.to_string(),
            input: input.to_string(),
            output: output.to_string(),
            technique: technique.map(|t| t.to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            hash: {
                let result = Md5::digest(input.as_bytes());
                result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            },
        };

        self.successes.push(record);

        // 更新统计
        self.stats.total += 1;
        *self.stats.categories.entry(category.to_string()).or_insert(0) += 1;

        // 提取模式
        let words: Vec<&str> = input.split_whitespace().collect();
        for word in words {
            *self.patterns.entry(word.to_string()).or_insert(0) += 1;
        }

        // 记录技术
        if let Some(tech) = technique {
            *self.techniques.entry(tech.to_string()).or_insert(0) += 1;
        }
    }

    /// 查询模式
    pub fn query_pattern(&self, pattern: &str) -> u32 {
        self.patterns.get(pattern).copied().unwrap_or(0)
    }

    /// 查询技术
    pub fn query_technique(&self, technique: &str) -> u32 {
        self.techniques.get(technique).copied().unwrap_or(0)
    }

    /// 获取最近的成功记录
    pub fn recent_successes(&self, n: usize) -> Vec<&SuccessRecord> {
        self.successes.iter().rev().take(n).collect()
    }

    /// 获取统计
    pub fn stats(&self) -> KernelStats {
        KernelStats {
            total_records: self.successes.len(),
            total_patterns: self.patterns.len(),
            total_techniques: self.techniques.len(),
            top_patterns: self.top_patterns(5),
            top_techniques: self.top_techniques(5),
        }
    }

    /// 获取热门模式
    fn top_patterns(&self, n: usize) -> Vec<(String, u32)> {
        let mut patterns: Vec<_> = self.patterns.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));
        patterns.into_iter().take(n).map(|(k, v)| (k.clone(), *v)).collect()
    }

    /// 获取热门技术
    fn top_techniques(&self, n: usize) -> Vec<(String, u32)> {
        let mut techniques: Vec<_> = self.techniques.iter().collect();
        techniques.sort_by(|a, b| b.1.cmp(a.1));
        techniques.into_iter().take(n).map(|(k, v)| (k.clone(), *v)).collect()
    }

    /// 保存到文件
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 从文件加载
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let kernel: Self = serde_json::from_str(&json)?;
        Ok(kernel)
    }
}

/// 内核统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelStats {
    pub total_records: usize,
    pub total_patterns: usize,
    pub total_techniques: usize,
    pub top_patterns: Vec<(String, u32)>,
    pub top_techniques: Vec<(String, u32)>,
}

impl std::fmt::Display for KernelStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "        MemoryKernel 统计")?;
        writeln!(f, "═══════════════════════════════════════════════")?;
        writeln!(f, "总记录:          {}", self.total_records)?;
        writeln!(f, "模式数:          {}", self.total_patterns)?;
        writeln!(f, "技术数:          {}", self.total_techniques)?;
        writeln!(f, "───────────────────────────────────────────────")?;
        writeln!(f, "热门模式:")?;
        for (pattern, count) in &self.top_patterns {
            writeln!(f, "  {}: {}", pattern, count)?;
        }
        writeln!(f, "热门技术:")?;
        for (technique, count) in &self.top_techniques {
            writeln!(f, "  {}: {}", technique, count)?;
        }
        writeln!(f, "═══════════════════════════════════════════════")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_kernel() {
        let mut kernel = MemoryKernel::new();
        kernel.learn_success("reasoning", "test input", "test output", Some("deduction"));
        
        assert_eq!(kernel.stats.total, 1);
        assert_eq!(kernel.query_pattern("test"), 1);
        assert_eq!(kernel.query_technique("deduction"), 1);
    }
}
