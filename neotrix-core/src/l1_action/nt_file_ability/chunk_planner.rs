//! 大表格智能分块器 (L3)
//!
//! 当表格超过 LLM 上下文窗口限制时，智能分块：
//! - 按行分块 (默认)
//! - 按列分块 (宽表格)
//! - 按语义分组 (有分组标记时)
//!
//! 设计原则:
//! - 每个 chunk 保留表头 (LLM 需要列名理解数据)
//! - chunk 大小可配置 (默认 4096 tokens ≈ 16K chars)
//! - 支持 overlap (重叠行，防止边界丢失上下文)

use super::types::TableData;

/// 分块配置
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// 每个 chunk 的最大字符数 (默认 16000 ≈ 4096 tokens)
    pub max_chars_per_chunk: usize,
    /// 重叠行数 (默认 2)
    pub overlap_rows: usize,
    /// 最大列数 (默认 20)
    pub max_cols: usize,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_chars_per_chunk: 16000,
            overlap_rows: 2,
            max_cols: 20,
        }
    }
}

/// 分块结果
#[derive(Debug, Clone)]
pub struct TableChunk {
    /// chunk 索引 (0-based)
    pub index: usize,
    /// 总 chunk 数
    pub total: usize,
    /// 包含的行范围 (start_row, end_row)
    pub row_range: (usize, usize),
    /// 分块后的表格数据
    pub table: TableData,
    /// 估算 token 数
    pub estimated_tokens: usize,
}

/// 大表格分块器
pub struct ChunkPlanner {
    config: ChunkConfig,
}

impl Default for ChunkPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl ChunkPlanner {
    pub fn new() -> Self {
        Self {
            config: ChunkConfig::default(),
        }
    }

    pub fn with_config(config: ChunkConfig) -> Self {
        Self { config }
    }

    /// 将大表格分块
    pub fn chunk(&self, table: &TableData) -> Vec<TableChunk> {
        let headers = &table.headers;
        let rows = &table.rows;
        let show_cols = headers.len().min(self.config.max_cols);
        let truncated_headers: Vec<String> = headers[..show_cols].to_vec();

        // 估算每行字符数
        let avg_row_chars = if rows.is_empty() {
            0
        } else {
            let total_chars: usize = rows
                .iter()
                .map(|r| r.iter().map(|c| c.len()).sum::<usize>())
                .sum();
            total_chars / rows.len()
        };

        // 计算每个 chunk 的行数
        let header_chars: usize = truncated_headers.iter().map(|h| h.len() + 3).sum();
        let available_chars = self
            .config
            .max_chars_per_chunk
            .saturating_sub(header_chars + 100); // 100 for formatting
        let rows_per_chunk = if avg_row_chars > 0 {
            (available_chars / avg_row_chars).max(1)
        } else {
            rows.len()
        };

        // 分块
        let mut chunks = Vec::new();
        let mut start = 0;
        let total_rows = rows.len();
        // 重叠不得超过步长，否则 start 会原地/倒退 → 死循环 + 内存无限增长（2026-09-27 实锤）
        let overlap = self.config.overlap_rows.min(rows_per_chunk.saturating_sub(1));

        while start < total_rows {
            let end = (start + rows_per_chunk).min(total_rows);
            let chunk_rows: Vec<Vec<String>> = rows[start..end]
                .iter()
                .map(|r| r[..show_cols].to_vec())
                .collect();

            let chunk_table = TableData {
                name: format!("{}_chunk_{}", table.name, chunks.len()),
                headers: truncated_headers.clone(),
                rows: chunk_rows,
            };

            let estimated_tokens = estimate_tokens(&chunk_table);

            chunks.push(TableChunk {
                index: chunks.len(),
                total: 0, // 稍后填充
                row_range: (start, end),
                table: chunk_table,
                estimated_tokens,
            });

            // 强制单调前进：end > start 恒成立，故此赋值必使 start 严格增长
            let next = end.saturating_sub(overlap);
            start = if next > start { next } else { end };
        }

        // 填充 total
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total = total;
        }

        chunks
    }
}

/// 估算 token 数 —— 复用本仓 **CJK 感知单一事实源**。
///
/// ⚠️ 2026-10-05 修单位混用 + 注释与实现不符。旧实现注释写「粗略: 1 token
/// ≈ 4 chars」，而代码是 `chars += h.len()`（**字节**）后 `/4`
/// ⇒ 中文表头/单元格被高估 3 倍。而本仓是中文为主，xlsx/csv 的中文表头
/// 与单元格是**常态**（本模块正是 xlsx/csv 读取结果的 chunk 规划器）
/// ⇒ `TableChunk.estimated_tokens` 系统性偏高，chunk 切分因此偏小。
///
/// ✅ 转出 `l1_action::nt_core_llm::estimate_tokens`（CJK 1 token/字、
/// 其余 4 字/token）。逐格累加会漏掉分隔符开销，故先拼总量再估。
fn estimate_tokens(table: &TableData) -> usize {
    let mut text = String::new();
    for h in &table.headers {
        text.push_str(h);
        text.push('\n');
    }
    for row in &table.rows {
        for cell in row {
            text.push_str(cell);
            text.push('\t');
        }
        text.push('\n');
    }
    crate::l1_action::nt_core_llm::estimate_tokens(&text)
}

/// 快捷函数: 自动分块
pub fn chunk_table(table: &TableData) -> Vec<TableChunk> {
    ChunkPlanner::new().chunk(table)
}

/// 快捷函数: 自动分块 (带配置)
pub fn chunk_table_with_config(table: &TableData, config: ChunkConfig) -> Vec<TableChunk> {
    ChunkPlanner::with_config(config).chunk(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_small_table() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["A".into(), "B".into()],
            rows: vec![vec!["1".into(), "2".into()], vec!["3".into(), "4".into()]],
        };
        let chunks = chunk_table(&table);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].total, 1);
    }

    #[test]
    fn test_chunk_large_table() {
        let mut rows = Vec::new();
        for i in 0..100 {
            rows.push(vec![format!("row_{}", i), format!("data_{}", i)]);
        }
        let table = TableData {
            name: "big".into(),
            headers: vec!["ID".into(), "Data".into()],
            rows,
        };
        let config = ChunkConfig {
            max_chars_per_chunk: 500,
            ..Default::default()
        };
        let chunks = ChunkPlanner::with_config(config).chunk(&table);
        assert!(chunks.len() > 1);
    }

    #[test]
    fn test_chunk_preserves_headers() {
        let mut rows = Vec::new();
        for i in 0..50 {
            rows.push(vec![format!("r{}", i), format!("v{}", i)]);
        }
        let table = TableData {
            name: "t".into(),
            headers: vec!["Key".into(), "Val".into()],
            rows,
        };
        let config = ChunkConfig {
            max_chars_per_chunk: 200,
            ..Default::default()
        };
        let chunks = ChunkPlanner::with_config(config).chunk(&table);
        for c in &chunks {
            assert_eq!(c.table.headers, vec!["Key", "Val"]);
        }
    }

    /// 回归（2026-09-27）：单行超长单元格 → rows_per_chunk=1，overlap=2 时
    /// 旧实现 start 原地踏步 → 死循环 + 内存无限增长，把测试进程吃到被内核 SIGKILL。
    #[test]
    fn test_chunk_forces_forward_progress() {
        let table = TableData {
            name: "wide".into(),
            headers: vec!["Blob".into()],
            rows: vec![
                vec!["x".repeat(500)],
                vec!["y".repeat(500)],
                vec!["z".repeat(500)],
            ],
        };
        let config = ChunkConfig {
            max_chars_per_chunk: 200,
            ..Default::default()
        };
        let chunks = ChunkPlanner::with_config(config).chunk(&table);
        assert!(
            chunks.len() <= 3,
            "chunk 数必须有界（否则死循环），实际 {}",
            chunks.len()
        );
        assert_eq!(chunks.last().map(|c| c.row_range.1), Some(3));
    }

    #[test]
    fn test_chunk_total_filled() {
        let mut rows = Vec::new();
        for i in 0..30 {
            rows.push(vec![format!("r{}", i)]);
        }
        let table = TableData {
            name: "t".into(),
            headers: vec!["C".into()],
            rows,
        };
        let config = ChunkConfig {
            max_chars_per_chunk: 100,
            ..Default::default()
        };
        let chunks = ChunkPlanner::with_config(config).chunk(&table);
        let total = chunks.len();
        for c in &chunks {
            assert_eq!(c.total, total);
        }
    }
}
