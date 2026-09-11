//! L3 表示层 — 表格→LLM 最优格式转换
//!
//! 逆向分析结论:
//! - CSV: 48 tokens/行 (最省)
//! - JSON: 111 tokens/行 (Function Calling 需要)
//! - Markdown: 67 tokens/行 (人类可读 + LLM 理解)
//!
//! 设计: 双通道输出
//! - Markdown 通道: 供 LLM 理解和推理
//! - JSON 通道: 供 Function Calling / Structured Output

use super::types::TableData;

/// LLM 表格呈现器
pub struct TablePresenter<'a> {
    table: &'a TableData,
    max_rows: usize,
    max_cols: usize,
}

impl<'a> TablePresenter<'a> {
    pub fn new(table: &'a TableData) -> Self {
        Self {
            table,
            max_rows: 50,
            max_cols: 20,
        }
    }

    pub fn with_limits(mut self, max_rows: usize, max_cols: usize) -> Self {
        self.max_rows = max_rows;
        self.max_cols = max_cols;
        self
    }

    /// Markdown 格式 (供 LLM 理解)
    pub fn to_markdown(&self) -> String {
        let headers = &self.table.headers;
        let rows = &self.table.rows;
        let show_rows = rows.len().min(self.max_rows);
        let show_cols = headers.len().min(self.max_cols);

        let mut md = String::new();

        // 表头
        md.push_str("| ");
        for h in &headers[..show_cols] {
            md.push_str(&format!("{} |", truncate(h, 20)));
        }
        md.push('\n');

        // 分隔线
        md.push_str("| ");
        for _ in 0..show_cols {
            md.push_str("--- |");
        }
        md.push('\n');

        // 数据行
        for row in &rows[..show_rows] {
            md.push_str("| ");
            for cell in &row[..show_cols] {
                md.push_str(&format!("{} |", truncate(cell, 20)));
            }
            md.push('\n');
        }

        // 截断提示
        if rows.len() > self.max_rows {
            md.push_str(&format!(
                "\n... (共 {} 行，显示前 {} 行)\n",
                rows.len(),
                self.max_rows
            ));
        }

        md
    }

    /// JSON 格式 (供 Function Calling)
    pub fn to_json(&self) -> serde_json::Value {
        let headers = &self.table.headers;
        let rows = &self.table.rows;
        let show_rows = rows.len().min(self.max_rows);
        let show_cols = headers.len().min(self.max_cols);

        let mut records = Vec::new();
        for row in &rows[..show_rows] {
            let mut record = serde_json::Map::new();
            for (i, h) in headers[..show_cols].iter().enumerate() {
                let value = row.get(i).map(|s| s.as_str()).unwrap_or("");
                record.insert(h.clone(), serde_json::Value::String(value.to_string()));
            }
            records.push(serde_json::Value::Object(record));
        }

        serde_json::json!({
            "table_name": self.table.name,
            "total_rows": rows.len(),
            "total_cols": headers.len(),
            "shown_rows": show_rows,
            "shown_cols": show_cols,
            "headers": headers[..show_cols].to_vec(),
            "records": records,
        })
    }

    /// CSV 格式 (最省 token)
    pub fn to_csv(&self) -> String {
        let headers = &self.table.headers;
        let rows = &self.table.rows;
        let show_rows = rows.len().min(self.max_rows);
        let show_cols = headers.len().min(self.max_cols);

        let mut csv = String::new();

        // 表头
        csv.push_str(&headers[..show_cols].join(","));
        csv.push('\n');

        // 数据行
        for row in &rows[..show_rows] {
            csv.push_str(&row[..show_cols].join(","));
            csv.push('\n');
        }

        csv
    }

    /// 统计摘要 (供 LLM 快速了解表格)
    pub fn summary(&self) -> String {
        let headers = &self.table.headers;
        let rows = &self.table.rows;
        let numeric_cols = count_numeric_columns(rows, headers.len());

        format!(
            "表格 '{}': {} 行 × {} 列。数值列: {}。表头: [{}]",
            self.table.name,
            rows.len(),
            headers.len(),
            numeric_cols,
            headers
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn count_numeric_columns(rows: &[Vec<String>], num_cols: usize) -> usize {
    let mut numeric_count = 0;
    for col in 0..num_cols {
        let is_numeric = rows
            .iter()
            .filter(|r| r.get(col).map(|c| !c.trim().is_empty()).unwrap_or(false))
            .all(|r| {
                r.get(col)
                    .map(|c| c.trim().replace(',', "").parse::<f64>().is_ok())
                    .unwrap_or(true)
            });
        if is_numeric {
            numeric_count += 1;
        }
    }
    numeric_count
}

/// 快捷函数: TableData → Markdown
pub fn table_to_markdown(table: &TableData) -> String {
    TablePresenter::new(table).to_markdown()
}

/// 快捷函数: TableData → JSON
pub fn table_to_json(table: &TableData) -> serde_json::Value {
    TablePresenter::new(table).to_json()
}

/// 快捷函数: TableData → 摘要
pub fn table_summary(table: &TableData) -> String {
    TablePresenter::new(table).summary()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_output() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["Name".into(), "Age".into()],
            rows: vec![
                vec!["Alice".into(), "30".into()],
                vec!["Bob".into(), "25".into()],
            ],
        };
        let md = TablePresenter::new(&table).to_markdown();
        assert!(md.contains("| Name | Age |"));
        assert!(md.contains("| Alice | 30 |"));
    }

    #[test]
    fn test_json_output() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["Name".into(), "Age".into()],
            rows: vec![vec!["Alice".into(), "30".into()]],
        };
        let json = TablePresenter::new(&table).to_json();
        assert_eq!(json["table_name"], "test");
        assert_eq!(json["total_rows"], 1);
        assert_eq!(json["records"][0]["Name"], "Alice");
    }

    #[test]
    fn test_csv_output() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["Name".into(), "Age".into()],
            rows: vec![vec!["Alice".into(), "30".into()]],
        };
        let csv = TablePresenter::new(&table).to_csv();
        assert!(csv.contains("Name,Age"));
        assert!(csv.contains("Alice,30"));
    }

    #[test]
    fn test_summary() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["Name".into(), "Age".into()],
            rows: vec![
                vec!["Alice".into(), "30".into()],
                vec!["Bob".into(), "25".into()],
            ],
        };
        let s = TablePresenter::new(&table).summary();
        assert!(s.contains("2 行"));
        assert!(s.contains("2 列"));
    }

    #[test]
    fn test_truncation() {
        let table = TableData {
            name: "test".into(),
            headers: vec!["A".into()],
            rows: (0..100).map(|i| vec![format!("row_{}", i)]).collect(),
        };
        let md = TablePresenter::new(&table).with_limits(5, 10).to_markdown();
        assert!(md.contains("共 100 行，显示前 5 行"));
    }
}
