//! # NT-IO context_sandbox — 工具输出沙箱压缩层
//!
//! 吸收源: context-mode pattern (GitHub 17K★)。
//! 所有工具输出经沙箱压缩后再进入 LLM 上下文窗口，
//! 原始输出存入 AddressableStore 以 §id 引用，摘要进入上下文。
//!
//! 典型压缩比: 315KB → 5.4KB (~58:1)。
//!
//! 骨架阶段 (C0): 三级压缩 + 按工具配置覆盖 + 历史统计。
//! 完善阶段 (C1): AddressableStore 实际接入、token 精确计数。

use std::collections::HashMap;
use crate::l1_action::nt_memory::addressable_store::AddressableStore;

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// 保留前 200 字符 + 后 100 字符
    Short,
    /// 保留前 500 字符 + 关键指标提取
    Medium,
    /// 保留完整输出（关键工具使用）
    Detailed,
}

/// 沙箱化后的工具输出
#[derive(Debug, Clone)]
pub struct SandboxedOutput {
    /// 工具名称
    pub tool_name: String,
    /// 原始输出
    pub raw_output: String,
    /// 压缩后的摘要
    pub summary: String,
    /// 原始 token 数估算
    pub original_tokens: usize,
    /// 摘要 token 数估算
    pub summary_tokens: usize,
    /// 压缩比 (原始/摘要)
    pub compression_ratio: f64,
    /// AddressableStore 引用 id（§id）
    pub citation_id: Option<String>,
}

/// 沙箱配置
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// 默认压缩级别
    pub default_level: CompressionLevel,
    /// 按工具名称的覆盖配置
    pub tool_overrides: HashMap<String, CompressionLevel>,
    /// 摘要最大 token 数
    pub max_summary_tokens: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            default_level: CompressionLevel::Medium,
            tool_overrides: HashMap::new(),
            max_summary_tokens: 200,
        }
    }
}

/// 上下文沙箱 — 在工具输出进入上下文前进行压缩
pub struct ContextSandbox {
    config: SandboxConfig,
    history: Vec<SandboxedOutput>,
    /// 追加式存储 — 原始输出按 §id 引用
    store: AddressableStore,
}

impl ContextSandbox {
    /// 创建新的上下文沙箱
    pub fn new(config: SandboxConfig) -> Self {
        Self {
            config,
            history: Vec::new(),
            store: AddressableStore::new(),
        }
    }

    /// 沙箱化工具输出: 压缩 + 存储 + 返回摘要
    pub fn sandbox(&mut self, tool_name: &str, output: &str) -> SandboxedOutput {
        let level = self.get_level(tool_name).clone();
        let summary = Self::compress(output, &level, self.config.max_summary_tokens);
        let original_tokens = Self::estimate_tokens(output);
        let summary_tokens = Self::estimate_tokens(&summary);
        let compression_ratio = if summary_tokens > 0 {
            original_tokens as f64 / summary_tokens as f64
        } else {
            1.0
        };

        // 接入 AddressableStore 生成 citation_id
        let citation_id = Some(self.store.append(tool_name, "", output));

        let out = SandboxedOutput {
            tool_name: tool_name.to_string(),
            raw_output: output.to_string(),
            summary,
            original_tokens,
            summary_tokens,
            compression_ratio,
            citation_id,
        };

        self.history.push(out.clone());
        out
    }

    /// 获取特定工具的压缩级别
    fn get_level(&self, tool_name: &str) -> &CompressionLevel {
        self.config
            .tool_overrides
            .get(tool_name)
            .unwrap_or(&self.config.default_level)
    }

    /// 按级别压缩输出
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    fn compress(output: &str, level: &CompressionLevel, max_tokens: usize) -> String {
        let max_chars = max_tokens * 4; // 粗估: 1 token ≈ 4 字符

        match level {
            CompressionLevel::Short => {
                if output.len() <= max_chars {
                    return output.to_string();
                }
                let head = &output[..output.len().min(200)];
                let tail = if output.len() > 300 {
                    &output[output.len() - 100..]
                } else {
                    ""
                };
                if tail.is_empty() {
                    format!("{head}…")
                } else {
                    format!("{head}…{tail}")
                }
            }
            CompressionLevel::Medium => {
                if output.len() <= max_chars {
                    return output.to_string();
                }
                let head = &output[..output.len().min(500)];

                // 提取关键指标: 包含数字/百分比/状态码的行
                let metrics: Vec<&str> = output
                    .lines()
                    .filter(|line| {
                        line.contains(|c: char| c.is_ascii_digit())
                            && (line.contains('%')
                                || line.contains("status")
                                || line.contains("error")
                                || line.contains("success")
                                || line.contains("failed")
                                || line.contains("count")
                                || line.contains("total")
                                || line.contains("bytes"))
                    })
                    .take(10)
                    .collect();

                if metrics.is_empty() {
                    format!("{head}…")
                } else {
                    format!("{head}\n\n--- 关键指标 ---\n{}", metrics.join("\n"))
                }
            }
            CompressionLevel::Detailed => output.to_string(),
        }
    }

    /// 粗估 token 数 (1 token ≈ 4 字符)
    fn estimate_tokens(text: &str) -> usize {
        text.len() / 4
    }

    /// 获取本会话所有沙箱化输出
    
    pub fn history(&self) -> &[SandboxedOutput] {
        &self.history
    }

    /// 所有沙箱化输出节省的总 token 数
    
    pub fn total_tokens_saved(&self) -> usize {
        self.history
            .iter()
            .map(|o| o.original_tokens.saturating_sub(o.summary_tokens))
            .sum()
    }

    /// 平均压缩比
    
    pub fn avg_compression(&self) -> f64 {
        if self.history.is_empty() {
            return 1.0;
        }
        let total: f64 = self.history.iter().map(|o| o.compression_ratio).sum();
        total / self.history.len() as f64
    }

    /// 通过 §id 召回完整原始输出
    pub fn recall(&self, citation_id: &str) -> Option<&str> {
        self.store.recall(citation_id).map(|obs| obs.output.as_str())
    }

    /// 获取 AddressableStore 引用
    pub fn store(&self) -> &AddressableStore {
        &self.store
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_compression() {
        let short_text = "short output";
        let result = ContextSandbox::compress(short_text, &CompressionLevel::Short, 200);
        assert_eq!(result, "short output");
    }

    #[test]
    fn test_medium_with_metrics() {
        let text = "Processing...\nstatus: 200\nTotal: 42 items\nDone.";
        let result = ContextSandbox::compress(text, &CompressionLevel::Medium, 200);
        assert!(result.contains("关键指标"));
    }

    #[test]
    fn test_detailed_passthrough() {
        let text = "full output here";
        let result = ContextSandbox::compress(text, &CompressionLevel::Detailed, 200);
        assert_eq!(result, "full output here");
    }

    #[test]
    fn test_sandbox_basic() {
        let config = SandboxConfig::default();
        let mut sandbox = ContextSandbox::new(config);
        let output = sandbox.sandbox("test_tool", "hello world");
        assert_eq!(output.tool_name, "test_tool");
        assert_eq!(output.summary, "hello world");
        assert_eq!(sandbox.history().len(), 1);
    }

    #[test]
    fn test_tool_override() {
        let mut config = SandboxConfig::default();
        config
            .tool_overrides
            .insert("critical_tool".to_string(), CompressionLevel::Detailed);
        let mut sandbox = ContextSandbox::new(config);
        let output = sandbox.sandbox("critical_tool", "important data");
        assert_eq!(output.summary, "important data");
    }

    #[test]
    fn test_tokens_saved() {
        let mut sandbox = ContextSandbox::new(SandboxConfig::default());
        let long = "x".repeat(4000);
        sandbox.sandbox("tool", &long);
        assert!(sandbox.total_tokens_saved() > 0);
    }
}
