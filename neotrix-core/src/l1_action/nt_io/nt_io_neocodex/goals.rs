// ── Goal System (from Kimi Code: chained goals) ──

use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum GoalState {
    Active,
    Paused,
    Completed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub state: GoalState,
    pub created_at: Instant,
    pub iterations: u64,
    pub max_iterations: u64,
}

pub struct GoalQueue {
    pub goals: VecDeque<Goal>,
    pub active: Option<Goal>,
    pub completed: Vec<Goal>,
    next_id: u64,
}

impl Default for GoalQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalQueue {
    pub fn new() -> Self {
        Self {
            goals: VecDeque::new(),
            active: None,
            completed: Vec::new(),
            next_id: 0,
        }
    }

    pub fn add(&mut self, description: &str, max_iterations: u64) {
        // completed.len() + goals.len() + 1 omits the active goal, so two
        // goals could share an id (e.g. add A -> g-1 active, add B -> g-1
        // again) and corrupt WireEvent::GoalUpdate correlation. Use a
        // monotonic counter instead.
        self.next_id += 1;
        let id = format!("g-{}", self.next_id);
        self.goals.push_back(Goal {
            id,
            description: description.to_string(),
            state: GoalState::Active,
            created_at: Instant::now(),
            iterations: 0,
            max_iterations,
        });
    }

    pub fn next(&mut self) -> Option<Goal> {
        if let Some(prev) = self.active.take() {
            self.completed.push(prev);
        }
        self.active = self.goals.pop_front();
        self.active.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_queue() {
        let mut queue = GoalQueue::new();
        queue.add("test goal 1", 3);
        queue.add("test goal 2", 5);
        assert!(queue.next().is_some());
        assert!(queue.next().is_some());
        assert!(queue.next().is_none());
    }
}