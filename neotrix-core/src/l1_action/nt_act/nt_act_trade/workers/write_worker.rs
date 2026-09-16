//! Write Worker — 文档/报告生成工作者
//!
//! 生成报价单、合同、装箱单、提单等贸易文档，以及各类业务报告。

#![forbid(unsafe_code)]

use std::time::Instant;

use async_trait::async_trait;

use super::{TradeWorker, WorkerResult, WorkerTask, WorkerType};

/// 文档/报告生成工作者
pub struct WriteWorker {
    worker_id: String,
}

impl WriteWorker {
    /// 创建文档生成工作者
    pub fn new(worker_id: &str) -> Self {
        Self {
            worker_id: worker_id.to_string(),
        }
    }
}

#[async_trait]
impl TradeWorker for WriteWorker {
    fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Write
    }

    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let start = Instant::now();

        let doc_type = task
            .params
            .get("document_type")
            .cloned()
            .unwrap_or_else(|| "report".to_string());

        let output_format = task
            .params
            .get("format")
            .cloned()
            .unwrap_or_else(|| "pdf".to_string());

        let data = serde_json::json!({
            "document_type": doc_type,
            "format": output_format,
            "status": "generated",
            "entity_id": task.entity_id,
        });

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(WorkerResult {
            task_id: task.task_id,
            success: true,
            data,
            duration_ms,
            error: None,
            metadata: [
                ("document_type".into(), doc_type),
                ("format".into(), output_format),
            ]
            .into_iter()
            .collect(),
        })
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(
            task_type,
            "write_quotation"
                | "write_contract"
                | "write_packing_list"
                | "write_bill_of_lading"
                | "write_report"
                | "write_invoice"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worker() -> WriteWorker {
        WriteWorker::new("write_test")
    }

    #[test]
    fn test_worker_id_and_type() {
        let w = make_worker();
        assert_eq!(w.worker_id(), "write_test");
        assert_eq!(w.worker_type(), WorkerType::Write);
    }

    #[test]
    fn test_can_handle() {
        let w = make_worker();
        assert!(w.can_handle("write_quotation"));
        assert!(w.can_handle("write_contract"));
        assert!(w.can_handle("write_report"));
        assert!(!w.can_handle("extract_orders"));
        assert!(!w.can_handle("send_email"));
    }

    #[tokio::test]
    async fn test_write_report() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "write_report".into(),
            entity_id: Some("ord_123".into()),
            params: [
                ("document_type".into(), "report".into()),
                ("format".into(), "pdf".into()),
            ]
            .into_iter()
            .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["document_type"], "report");
        assert_eq!(result.data["format"], "pdf");
    }

    #[tokio::test]
    async fn test_write_quotation() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t2".into(),
            task_type: "write_quotation".into(),
            entity_id: Some("q_456".into()),
            params: HashMap::new(),
            priority: 80,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert!(result.metadata.contains_key("document_type"));
    }
}
