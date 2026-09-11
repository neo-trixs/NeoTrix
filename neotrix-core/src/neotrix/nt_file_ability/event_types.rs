//! Excel 解析能力 EventBus 事件类型
//!
//! 事件驱动通信:
//! - XlsxParseStarted: 解析开始
//! - XlsxParseCompleted: 解析完成
//! - XlsxParseFailed: 解析失败
//! - TemplateDetected: 模板检测完成
//! - ConfigParsed: 配置解析完成

use serde::{Deserialize, Serialize};

/// Excel 解析事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExcelEvent {
    /// 解析开始
    XlsxParseStarted {
        file_path: String,
        parse_mode: String,
    },
    /// 解析完成
    XlsxParseCompleted {
        file_path: String,
        total_rows: usize,
        total_cols: usize,
        duration_ms: u64,
    },
    /// 解析失败
    XlsxParseFailed {
        file_path: String,
        error: String,
    },
    /// 模板检测完成
    TemplateDetected {
        file_path: String,
        template_type: String,
        confidence: f64,
    },
    /// 配置解析完成
    ConfigParsed {
        raw: String,
        fields_count: usize,
    },
}

impl ExcelEvent {
    /// 事件名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::XlsxParseStarted { .. } => "xlsx_parse_started",
            Self::XlsxParseCompleted { .. } => "xlsx_parse_completed",
            Self::XlsxParseFailed { .. } => "xlsx_parse_failed",
            Self::TemplateDetected { .. } => "template_detected",
            Self::ConfigParsed { .. } => "config_parsed",
        }
    }

    /// 事件数据 (JSON)
    pub fn data(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

/// 事件处理器 trait
pub trait ExcelEventHandler {
    fn handle_event(&self, event: &ExcelEvent);
}

/// 默认事件处理器 (日志输出)
pub struct LogExcelEventHandler;

impl ExcelEventHandler for LogExcelEventHandler {
    fn handle_event(&self, event: &ExcelEvent) {
        match event {
            ExcelEvent::XlsxParseStarted {
                file_path,
                parse_mode,
            } => {
                tracing::info!("Excel 解析开始: {} (模式: {})", file_path, parse_mode);
            }
            ExcelEvent::XlsxParseCompleted {
                file_path,
                total_rows,
                total_cols,
                duration_ms,
            } => {
                tracing::info!(
                    "Excel 解析完成: {} ({} 行 × {} 列, {}ms)",
                    file_path,
                    total_rows,
                    total_cols,
                    duration_ms
                );
            }
            ExcelEvent::XlsxParseFailed { file_path, error } => {
                tracing::error!("Excel 解析失败: {} - {}", file_path, error);
            }
            ExcelEvent::TemplateDetected {
                file_path,
                template_type,
                confidence,
            } => {
                tracing::info!(
                    "模板检测: {} -> {} (置信度: {:.2})",
                    file_path,
                    template_type,
                    confidence
                );
            }
            ExcelEvent::ConfigParsed { raw, fields_count } => {
                tracing::debug!("配置解析: {} ({} 个字段)", raw, fields_count);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_names() {
        let started = ExcelEvent::XlsxParseStarted {
            file_path: "test.xlsx".into(),
            parse_mode: "Auto".into(),
        };
        assert_eq!(started.name(), "xlsx_parse_started");

        let completed = ExcelEvent::XlsxParseCompleted {
            file_path: "test.xlsx".into(),
            total_rows: 10,
            total_cols: 5,
            duration_ms: 123,
        };
        assert_eq!(completed.name(), "xlsx_parse_completed");

        let failed = ExcelEvent::XlsxParseFailed {
            file_path: "bad.xlsx".into(),
            error: "parse error".into(),
        };
        assert_eq!(failed.name(), "xlsx_parse_failed");

        let template = ExcelEvent::TemplateDetected {
            file_path: "t.xlsx".into(),
            template_type: "采购申请单".into(),
            confidence: 0.95,
        };
        assert_eq!(template.name(), "template_detected");

        let config = ExcelEvent::ConfigParsed {
            raw: "压力:150LB".into(),
            fields_count: 1,
        };
        assert_eq!(config.name(), "config_parsed");
    }

    #[test]
    fn test_event_serialization() {
        let event = ExcelEvent::XlsxParseStarted {
            file_path: "test.xlsx".into(),
            parse_mode: "Auto".into(),
        };
        let json = event.data();
        assert_eq!(json["file_path"], "test.xlsx");
        assert_eq!(json["parse_mode"], "Auto");
    }
}
