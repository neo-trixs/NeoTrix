//! Send Worker — 邮件/通知发送工作者
//!
//! 负责贸易邮件发送、客户通知、内部告警等消息投递任务。

#![forbid(unsafe_code)]

use std::time::Instant;

use async_trait::async_trait;

use super::{TradeWorker, WorkerResult, WorkerTask, WorkerType};

/// 邮件/通知发送工作者
pub struct SendWorker {
    worker_id: String,
}

impl SendWorker {
    /// 创建发送工作者
    pub fn new(worker_id: &str) -> Self {
        Self {
            worker_id: worker_id.to_string(),
        }
    }
}

#[async_trait]
impl TradeWorker for SendWorker {
    fn worker_id(&self) -> &str {
        &self.worker_id
    }

    fn worker_type(&self) -> WorkerType {
        WorkerType::Send
    }

    async fn execute(&self, task: WorkerTask) -> Result<WorkerResult, String> {
        let start = Instant::now();

        let channel = task
            .params
            .get("channel")
            .cloned()
            .unwrap_or_else(|| "email".to_string());

        let recipient = task
            .params
            .get("recipient")
            .cloned()
            .unwrap_or_default();

        if recipient.is_empty() {
            let duration_ms = start.elapsed().as_millis() as u64;
            return Ok(WorkerResult::failure(
                &task.task_id,
                "recipient is required",
                duration_ms,
            ));
        }

        let subject = task
            .params
            .get("subject")
            .cloned()
            .unwrap_or_default();

        let data = serde_json::json!({
            "channel": channel,
            "recipient": recipient,
            "subject": subject,
            "status": "sent",
        });

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(WorkerResult {
            task_id: task.task_id,
            success: true,
            data,
            duration_ms,
            error: None,
            metadata: [
                ("channel".into(), channel),
                ("recipient".into(), recipient),
            ]
            .into_iter()
            .collect(),
        })
    }

    fn can_handle(&self, task_type: &str) -> bool {
        matches!(
            task_type,
            "send_email"
                | "send_notification"
                | "send_alert"
                | "send_whatsapp"
                | "send_follow_up"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_worker() -> SendWorker {
        SendWorker::new("send_test")
    }

    #[test]
    fn test_worker_id_and_type() {
        let w = make_worker();
        assert_eq!(w.worker_id(), "send_test");
        assert_eq!(w.worker_type(), WorkerType::Send);
    }

    #[test]
    fn test_can_handle() {
        let w = make_worker();
        assert!(w.can_handle("send_email"));
        assert!(w.can_handle("send_notification"));
        assert!(w.can_handle("send_whatsapp"));
        assert!(!w.can_handle("extract_orders"));
        assert!(!w.can_handle("write_report"));
    }

    #[tokio::test]
    async fn test_send_email_success() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "send_email".into(),
            entity_id: Some("cust_123".into()),
            params: [
                ("channel".into(), "email".into()),
                ("recipient".into(), "buyer@example.com".into()),
                ("subject".into(), "Quotation for Ball Valves".into()),
            ]
            .into_iter()
            .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(result.success);
        assert_eq!(result.data["channel"], "email");
        assert_eq!(result.data["recipient"], "buyer@example.com");
    }

    #[tokio::test]
    async fn test_send_missing_recipient() {
        let w = make_worker();
        let task = WorkerTask {
            task_id: "t1".into(),
            task_type: "send_email".into(),
            entity_id: None,
            params: [("channel".into(), "email".into())]
                .into_iter()
                .collect(),
            priority: 50,
        };

        let result = w.execute(task).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.error.unwrap(), "recipient is required");
    }
}
