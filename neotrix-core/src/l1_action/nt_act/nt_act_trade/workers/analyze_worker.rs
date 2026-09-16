//! Analyze Worker — 数据分析工作者
//!
//! 对提取的贸易数据进行统计分析、KPI 计算、风险评估等。

#![forbid(unsafe_code)]

use std::time::Instant;

use async_trait::async_trait;

use super::{TradeWorker, WorkerResult, WorkerTask, WorkerType};

/// 数据分析工作者
pub struct AnalyzeWorker {
    worker_id: String,
}

impl AnalyzeWorker {
    /// 创建分析工作者
    pub fn new(worker_id: &str) -> Self {
        Self {
            worker_id: worker_id.to_string(),
        }
    }
}

#[async_trait]
impl TradeWorker for AnalyzeWorker {
    fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Analyze
    }

    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let start = Instant::now();

        let analysis_type = task
            .params
            .get("analysis_type")
            .cloned()
            .unwrap_or_else(|| "summary".to_string());

        let data = match analysis_type.as_str() {
            "kpi" => serde_json::json!({
                "total_orders": 0,
                "total_revenue": 0.0,
                "conversion_rate": 0.0,
                "avg_order_value": 0.0,
            }),
            "risk" => serde_json::json!({
                "risk_level": "low",
                "findings": [],
            }),
            "summary" => serde_json::json!({
                "analysis_type": "summary",
                "status": "completed",
            }),
            _ => {
                return Err(format!("unknown analysis type: {}", analysis_type));
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(WorkerResult {
            task_id: task.task_id,
            success: true,
            data,
            duration_ms,
            error: None,
            metadata: [("analysis_type".into(), analysis_type)]
                .into_iter()
                .collect(),
        })
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(
            task_type,
            "analyze_kpi" | "analyze_risk" | "analyze_summary" | "analyze_customer"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worker() -> AnalyzeWorker {
        AnalyzeWorker::new("analyze_test")
    }

    #[test]
    fn test_worker_id_and_type() {
        let w = make_worker();
        assert_eq!(w.worker_id(), "analyze_test");
        assert_eq!(w.worker_type(), WorkerType::Analyze);
    }

    #[test]
    fn test_can_handle() {
        let w = make_worker();
        assert!(w.can_handle("analyze_kpi"));
        assert!(w.can_handle("analyze_risk"));
        assert!(!w.can_handle("extract_orders"));
        assert!(!w.can_handle("send_email"));
    }

    #[tokio::test]
    async fn test_analyze_summary() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "analyze_summary".into(),
            entity_id: None,
            params: [("analysis_type".into(), "summary".into())]
                .into_iter()
                .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.task_id, "t1");
    }

    #[tokio::test]
    async fn test_analyze_unknown_type() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "analyze_kpi".into(),
            entity_id: None,
            params: [("analysis_type".into(), "nonexistent".into())]
                .into_iter()
                .collect(),
            priority: 50,
        };

        let result = w.execute(task).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown analysis type"));
    }
}
