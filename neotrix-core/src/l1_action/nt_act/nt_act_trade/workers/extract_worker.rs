//! Extract Worker — 数据提取工作者
//!
//! 从外部平台 (阿里巴巴、中国制造网等) 或文件中提取贸易数据，
//! 委托 `TradeDataPipeline` 执行实际提取逻辑。

#![forbid(unsafe_code)]

use std::time::Instant;

use async_trait::async_trait;

use super::{TradeWorker, WorkerResult, WorkerTask, WorkerType};
use crate::l1_action::nt_act::nt_act_trade::data_pipeline::TradeDataPipeline;

/// 数据提取工作者
pub struct ExtractWorker {
    worker_id: String,
    pipeline: TradeDataPipeline,
}

impl ExtractWorker {
    /// 创建提取工作者
    pub fn new(worker_id: &str, pipeline: TradeDataPipeline) -> Self {
        Self {
            worker_id: worker_id.to_string(),
            pipeline,
        }
    }

    /// 获取底层 pipeline 引用
    pub fn pipeline(&self) -> &TradeDataPipeline {
        &self.pipeline
    }
}

#[async_trait]
impl TradeWorker for ExtractWorker {
    fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Extract
    }

    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let start = Instant::now();

        let platform = task
            .params
            .get("platform")
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let result = self.pipeline.extract_all(&platform).await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        let data = serde_json::json!({
            "customers_count": result.customers.len(),
            "emails_count": result.emails.len(),
            "interactions_count": result.interactions.len(),
            "duration_ms": result.duration_ms,
        });

        Ok(WorkerResult {
            task_id: task.task_id,
            success: true,
            data,
            duration_ms,
            error: None,
            metadata: [("platform".into(), platform)].into_iter().collect(),
        })
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(
            task_type,
            "extract_orders"
                | "extract_customers"
                | "extract_emails"
                | "extract_interactions"
                | "sync_platform"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l1_action::nt_act::nt_act_trade::data_pipeline::PlatformRegistry;

    fn make_worker() -> ExtractWorker {
        let registry = PlatformRegistry::new();
        let pipeline = TradeDataPipeline::with_registry(registry);
        ExtractWorker::new("extract_test", pipeline)
    }

    #[test]
    fn test_worker_id_and_type() {
        let w = make_worker();
        assert_eq!(w.worker_id(), "extract_test");
        assert_eq!(w.worker_type(), WorkerType::Extract);
    }

    #[test]
    fn test_can_handle() {
        let w = make_worker();
        assert!(w.can_handle("extract_orders"));
        assert!(w.can_handle("extract_customers"));
        assert!(w.can_handle("sync_platform"));
        assert!(!w.can_handle("analyze_kpi"));
        assert!(!w.can_handle("send_email"));
    }

    #[tokio::test]
    async fn test_extract_unknown_platform_fails() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "extract_orders".into(),
            entity_id: None,
            params: [("platform".into(), "nonexistent".into())].into_iter().collect(),
            priority: 50,
        };

        let result = w.execute(task).await;
        assert!(result.is_err());
    }
}
