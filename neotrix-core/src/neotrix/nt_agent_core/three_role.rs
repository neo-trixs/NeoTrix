#![allow(dead_code)]
use std::collections::HashMap;

/// The three core roles in verifiable computation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComputationRole {
    /// Requests a computation to be performed
    Requester,
    /// Performs the computation
    Provider,
    /// Verifies the computation result
    Verifier,
}

/// A computation task with role separation
#[derive(Debug, Clone)]
pub struct RoleSeparatedTask {
    pub task_id: u64,
    pub description: String,
    pub requester: String,
    pub provider: Option<String>,
    pub verifier: Option<String>,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub verification_passed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum TaskStatus {
    Requested,
    Assigned,
    Completed,
    Verified,
    Failed(String),
}

/// Manages role-separated computation lifecycle
pub struct ThreeRoleManager {
    tasks: HashMap<u64, RoleSeparatedTask>,
    next_id: u64,
    /// How many distinct agents must verify (default 1)
    verification_quorum: usize,
}

impl ThreeRoleManager {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            verification_quorum: 1,
        }
    }

    pub fn with_verification_quorum(quorum: usize) -> Self {
        Self {
            tasks: HashMap::new(),
            next_id: 1,
            verification_quorum: quorum,
        }
    }

    /// Requester submits a task
    pub fn submit_task(&mut self, requester: &str, description: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.insert(
            id,
            RoleSeparatedTask {
                task_id: id,
                description: description.to_string(),
                requester: requester.to_string(),
                provider: None,
                verifier: None,
                status: TaskStatus::Requested,
                result: None,
                verification_passed: None,
            },
        );
        id
    }

    /// Assign a provider to work on a task
    pub fn assign_provider(&mut self, task_id: u64, provider: &str) -> Result<(), String> {
        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.status != TaskStatus::Requested {
            return Err("Task not in Requested state".into());
        }
        task.provider = Some(provider.to_string());
        task.status = TaskStatus::Assigned;
        Ok(())
    }

    /// Provider submits a result
    pub fn submit_result(
        &mut self,
        task_id: u64,
        provider: &str,
        result: &str,
    ) -> Result<(), String> {
        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.provider.as_deref() != Some(provider) {
            return Err("Wrong provider".into());
        }
        if task.status != TaskStatus::Assigned {
            return Err("Task not in Assigned state".into());
        }
        task.result = Some(result.to_string());
        task.status = TaskStatus::Completed;
        Ok(())
    }

    /// Assign a verifier (different from provider)
    pub fn assign_verifier(&mut self, task_id: u64, verifier: &str) -> Result<(), String> {
        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.provider.as_deref() == Some(verifier) {
            return Err("Verifier cannot be the same as provider".into());
        }
        task.verifier = Some(verifier.to_string());
        Ok(())
    }

    /// Verifier submits verification result
    pub fn verify_result(
        &mut self,
        task_id: u64,
        verifier: &str,
        passed: bool,
    ) -> Result<(), String> {
        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.verifier.as_deref() != Some(verifier) {
            return Err("Wrong verifier".into());
        }
        if task.status != TaskStatus::Completed {
            return Err("Task not completed yet".into());
        }
        task.verification_passed = Some(passed);
        task.status = if passed {
            TaskStatus::Verified
        } else {
            TaskStatus::Failed("Verification failed".into())
        };
        Ok(())
    }

    /// Get task status for reporting
    pub fn status_summary(&self) -> Vec<(u64, String, String)> {
        self.tasks
            .values()
            .map(|t| (t.task_id, t.description.clone(), format!("{:?}", t.status)))
            .collect()
    }

    pub fn verified_count(&self) -> usize {
        self.tasks
            .values()
            .filter(|t| t.status == TaskStatus::Verified)
            .count()
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn get_task(&self, task_id: u64) -> Option<&RoleSeparatedTask> {
        self.tasks.get(&task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_lifecycle() {
        let mut manager = ThreeRoleManager::new();

        let id = manager.submit_task("alice", "compute hash");
        assert!(manager.task_count() == 1);

        manager.assign_provider(id, "bob").unwrap();
        manager.submit_result(id, "bob", "0xdeadbeef").unwrap();

        manager.assign_verifier(id, "carol").unwrap();
        manager.verify_result(id, "carol", true).unwrap();

        let task = manager.get_task(id).unwrap();
        assert!(matches!(task.status, TaskStatus::Verified));
        assert_eq!(task.verification_passed, Some(true));
        assert_eq!(manager.verified_count(), 1);
    }

    #[test]
    fn test_provider_cannot_be_verifier() {
        let mut manager = ThreeRoleManager::new();

        let id = manager.submit_task("alice", "compute hash");
        manager.assign_provider(id, "bob").unwrap();
        manager.submit_result(id, "bob", "result").unwrap();

        let err = manager.assign_verifier(id, "bob").unwrap_err();
        assert!(err.contains("cannot be the same"));
    }

    #[test]
    fn test_wrong_state_transitions_rejected() {
        let mut manager = ThreeRoleManager::new();

        let id = manager.submit_task("alice", "task");

        // can't submit result before assignment
        let err = manager.submit_result(id, "bob", "result").unwrap_err();
        assert!(err.contains("not in Assigned"));

        manager.assign_provider(id, "bob").unwrap();

        // can't assign provider again
        let err = manager.assign_provider(id, "charlie").unwrap_err();
        assert!(err.contains("not in Requested"));

        // can't verify before completion
        manager.assign_verifier(id, "carol").unwrap();
        let err = manager.verify_result(id, "carol", true).unwrap_err();
        assert!(err.contains("not completed"));
    }

    #[test]
    fn test_status_summary() {
        let mut manager = ThreeRoleManager::new();

        let id1 = manager.submit_task("alice", "task 1");
        let id2 = manager.submit_task("bob", "task 2");

        manager.assign_provider(id1, "carol").unwrap();
        manager.submit_result(id1, "carol", "ok").unwrap();
        manager.assign_verifier(id1, "dave").unwrap();
        manager.verify_result(id1, "dave", true).unwrap();

        let summary = manager.status_summary();
        assert_eq!(summary.len(), 2);

        let (_tid, desc, status) = summary.iter().find(|(id, _, _)| *id == id1).unwrap();
        assert_eq!(desc, "task 1");
        assert_eq!(status, "Verified");

        let (_, desc2, status2) = summary.iter().find(|(id, _, _)| *id == id2).unwrap();
        assert_eq!(desc2, "task 2");
        assert_eq!(status2, "Requested");
    }

    #[test]
    fn test_multiple_concurrent_tasks() {
        let mut manager = ThreeRoleManager::new();

        let ids: Vec<u64> = (0..5)
            .map(|i| manager.submit_task("requester", &format!("task {}", i)))
            .collect();

        for (i, id) in ids.iter().enumerate() {
            let provider = format!("provider_{}", i);
            let verifier = format!("verifier_{}", i);
            manager.assign_provider(*id, &provider).unwrap();
            manager.submit_result(*id, &provider, "result").unwrap();
            manager.assign_verifier(*id, &verifier).unwrap();
            manager.verify_result(*id, &verifier, true).unwrap();
        }

        assert_eq!(manager.verified_count(), 5);
        assert_eq!(manager.task_count(), 5);

        let summary = manager.status_summary();
        assert_eq!(summary.len(), 5);
        for (_id, _desc, status) in &summary {
            assert_eq!(status, "Verified");
        }
    }

    #[test]
    fn test_verification_failure() {
        let mut manager = ThreeRoleManager::new();

        let id = manager.submit_task("alice", "critical compute");
        manager.assign_provider(id, "bob").unwrap();
        manager.submit_result(id, "bob", "wrong result").unwrap();
        manager.assign_verifier(id, "carol").unwrap();
        manager.verify_result(id, "carol", false).unwrap();

        let task = manager.get_task(id).unwrap();
        assert!(matches!(task.status, TaskStatus::Failed(_)));
        assert_eq!(task.verification_passed, Some(false));
        assert_eq!(manager.verified_count(), 0);
    }
}
