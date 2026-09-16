//! Trade Tasks & Calendar — 外贸任务/日历/跟进提醒模块
//!
//! 对标 TMS 平台的任务管理能力：
//! - 任务创建/分配/完成
//! - 跟进提醒 (客户跟进/报价到期/付款到期)
//! - 日历视图 (按天/周/月)
//! - 任务优先级 + 截止日期
//! - 任务与客户/订单关联

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// 1. 任务类型与优先级
// ============================================================

/// 任务类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TradeTaskType {
    /// 客户跟进
    CustomerFollowUp,
    /// 报价跟进
    QuoteFollowUp,
    /// 付款提醒
    PaymentReminder,
    /// 发货跟进
    ShipmentFollowUp,
    /// 样品寄送
    SampleShipment,
    /// 合同签署
    ContractSigning,
    /// 生产催货
    ProductionChase,
    /// 单证准备
    DocumentPreparation,
    /// 其他
    Other(String),
}

/// 任务优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Urgent = 4,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "低"),
            Self::Medium => write!(f, "中"),
            Self::High => write!(f, "高"),
            Self::Urgent => write!(f, "紧急"),
        }
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TradeTaskStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
    Overdue,
}

// ============================================================
// 2. 任务记录
// ============================================================

/// 任务记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeTask {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub task_type: TradeTaskType,
    pub priority: TaskPriority,
    pub status: TradeTaskStatus,
    /// 关联客户 ID
    pub customer_id: Option<String>,
    /// 关联订单 ID
    pub order_id: Option<String>,
    /// 关联联系人 ID
    pub contact_id: Option<String>,
    /// 负责人 ID
    pub assignee_id: String,
    /// 截止时间
    pub due_at: Option<u64>,
    /// 完成时间
    pub completed_at: Option<u64>,
    /// 提醒时间
    pub remind_at: Option<u64>,
    /// 是否已提醒
    pub reminded: bool,
    /// 备注
    pub notes: Vec<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 日历事件 (任务的日历视图)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub task_id: String,
    pub title: String,
    pub date: String, // YYYY-MM-DD
    pub all_day: bool,
    pub start_time: Option<String>, // HH:MM
    pub end_time: Option<String>,
    pub task_type: TradeTaskType,
    pub priority: TaskPriority,
    pub status: TradeTaskStatus,
}

// ============================================================
// 3. 任务引擎
// ============================================================

/// 任务/日历引擎
pub struct TradeTaskEngine {
    tasks: HashMap<String, TradeTask>,
    /// 按客户索引任务
    customer_index: HashMap<String, Vec<String>>,
    /// 按订单索引任务
    order_index: HashMap<String, Vec<String>>,
}

impl Default for TradeTaskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TradeTaskEngine {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            customer_index: HashMap::new(),
            order_index: HashMap::new(),
        }
    }

    /// 创建任务
    pub fn create_task(
        &mut self,
        title: &str,
        task_type: TradeTaskType,
        priority: TaskPriority,
        assignee_id: &str,
        due_at: Option<u64>,
        customer_id: Option<String>,
        order_id: Option<String>,
    ) -> TradeTask {
        let now = self.current_timestamp();
        let task = TradeTask {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: None,
            task_type,
            priority,
            status: TradeTaskStatus::Pending,
            customer_id: customer_id.clone(),
            order_id: order_id.clone(),
            contact_id: None,
            assignee_id: assignee_id.to_string(),
            due_at,
            completed_at: None,
            remind_at: due_at.map(|d| d - 3600), // 提前1小时提醒
            reminded: false,
            notes: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        if let Some(ref cid) = customer_id {
            self.customer_index
                .entry(cid.clone())
                .or_default()
                .push(task.id.clone());
        }
        if let Some(ref oid) = order_id {
            self.order_index
                .entry(oid.clone())
                .or_default()
                .push(task.id.clone());
        }
        self.tasks.insert(task.id.clone(), task.clone());
        task
    }

    /// 完成任务
    pub fn complete_task(&mut self, task_id: &str) -> Result<(), String> {
        let now = self.current_timestamp();
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| format!("Task {} not found", task_id))?;
        task.status = TradeTaskStatus::Completed;
        task.completed_at = Some(now);
        task.updated_at = now;
        Ok(())
    }

    /// 获取待办任务
    pub fn pending_tasks(&self, assignee_id: Option<&str>) -> Vec<&TradeTask> {
        self.tasks
            .values()
            .filter(|t| {
                t.status == TradeTaskStatus::Pending || t.status == TradeTaskStatus::InProgress
            })
            .filter(|t| assignee_id.map_or(true, |a| t.assignee_id == a))
            .collect()
    }

    /// 获取逾期任务
    pub fn overdue_tasks(&self) -> Vec<&TradeTask> {
        let now = self.current_timestamp();
        self.tasks
            .values()
            .filter(|t| {
                (t.status == TradeTaskStatus::Pending || t.status == TradeTaskStatus::InProgress)
                    && t.due_at.map_or(false, |d| d < now)
            })
            .collect()
    }

    /// 获取需要提醒的任务
    pub fn due_reminders(&mut self) -> Vec<&TradeTask> {
        let now = self.current_timestamp();
        let due_ids: Vec<String> = self
            .tasks
            .values()
            .filter(|t| {
                !t.reminded
                    && t.status != TradeTaskStatus::Completed
                    && t.remind_at.map_or(false, |r| r <= now)
            })
            .map(|t| t.id.clone())
            .collect();
        // 标记已提醒
        for id in &due_ids {
            if let Some(t) = self.tasks.get_mut(id) {
                t.reminded = true;
            }
        }
        due_ids.iter().filter_map(|id| self.tasks.get(id)).collect()
    }

    /// 生成日历视图 (指定月份)
    pub fn calendar_view(&self, year: u32, month: u32) -> Vec<CalendarEvent> {
        self.tasks
            .values()
            .filter(|t| {
                if let Some(due) = t.due_at {
                    let dt = chrono::DateTime::from_timestamp(due as i64, 0);
                    if let Some(dt) = dt {
                        dt.format("%Y-%m").to_string()
                            == format!("{:04}-{:02}", year, month)
                    } else {
                        false
                    }
                } else {
                    false
                }
            })
            .map(|t| {
                let due_str = t.due_at.map(|d| {
                    chrono::DateTime::from_timestamp(d as i64, 0)
                        .unwrap_or_default()
                        .format("%Y-%m-%d")
                        .to_string()
                });
                CalendarEvent {
                    task_id: t.id.clone(),
                    title: t.title.clone(),
                    date: due_str.unwrap_or_default(),
                    all_day: true,
                    start_time: None,
                    end_time: None,
                    task_type: t.task_type.clone(),
                    priority: t.priority,
                    status: t.status.clone(),
                }
            })
            .collect()
    }

    fn current_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

// ============================================================
// 4. 测试
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_complete_task() {
        let mut engine = TradeTaskEngine::new();
        let task = engine.create_task(
            "Follow up with Acme",
            TradeTaskType::CustomerFollowUp,
            TaskPriority::High,
            "sales-01",
            None,
            Some("CUST-001".into()),
            None,
        );
        assert_eq!(task.status, TradeTaskStatus::Pending);
        engine.complete_task(&task.id).unwrap();
        let task = engine.tasks.get(&task.id).unwrap();
        assert_eq!(task.status, TradeTaskStatus::Completed);
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_pending_tasks() {
        let mut engine = TradeTaskEngine::new();
        engine.create_task("Task 1", TradeTaskType::CustomerFollowUp, TaskPriority::High, "s1", None, None, None);
        engine.create_task("Task 2", TradeTaskType::QuoteFollowUp, TaskPriority::Low, "s1", None, None, None);
        let pending = engine.pending_tasks(None);
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_overdue_tasks() {
        let mut engine = TradeTaskEngine::new();
        // 创建一个已过期的任务 (due_at = 0)
        engine.create_task(
            "Overdue task",
            TradeTaskType::PaymentReminder,
            TaskPriority::Urgent,
            "s1",
            Some(0), // 已过期
            None,
            None,
        );
        let overdue = engine.overdue_tasks();
        assert_eq!(overdue.len(), 1);
    }
}
