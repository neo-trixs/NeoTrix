//! 并行模块测试

use crate::neotrix::parallel::types::{Agent, Task, AgentPool};
use crate::neotrix::parallel::executor::{ParallelExecutor, ExecMode};
use crate::neotrix::parallel::hands::{HandsController, HandType};

#[test]
fn test_agent_creation() {
    let agent = Agent::new("test-agent".to_string());
    assert_eq!(agent.id, "test-agent");
    assert!(!agent.busy);
}

#[test]
fn test_agent_assign() {
    let mut agent = Agent::new("test".to_string());
    agent.assign_task(1);
    assert!(agent.busy);
    assert_eq!(agent.current_task, Some(1));
}

#[test]
fn test_pool_assign() {
    let mut pool = AgentPool::new(4);
    pool.register(Agent::new("a1".to_string()));
    pool.register(Agent::new("a2".to_string()));

    let task = Task::new("a1".to_string(), vec![1.0; 256], 0);
    let assigned = pool.assign(&task);

    assert!(assigned.is_some());
}

#[tokio::test]
async fn test_sequential_exec() {
    let mut exec = ParallelExecutor::new(2);
    exec.set_mode(ExecMode::Sequential);
    exec.add_task("a1".to_string(), vec![1.0, 2.0, 3.0, 4.0], 0);
    exec.add_task("a2".to_string(), vec![4.0, 3.0, 2.0, 1.0], 0);

    let results = exec.execute().await;
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_parallel_exec() {
    let mut exec = ParallelExecutor::new(2);
    exec.set_mode(ExecMode::Parallel);
    exec.add_task("a1".to_string(), vec![1.0, 2.0], 0);
    exec.add_task("a2".to_string(), vec![3.0, 4.0], 0);

    let results = exec.execute().await;
    assert_eq!(results.len(), 2);
}

#[test]
fn test_hands_creation() {
    let mut controller = HandsController::new(4);
    controller.register_hand(HandType::Browser);
    assert!(controller.acquire(HandType::Browser).is_some());
    controller.release(HandType::Browser);
    assert!(controller.acquire(HandType::Browser).is_some());
}

#[test]
fn test_all_hand_types() {
    let mut controller = HandsController::new(7);
    controller.register_hand(HandType::Browser);
    controller.register_hand(HandType::Terminal);
    controller.register_hand(HandType::FileSystem);
    controller.register_hand(HandType::CodeEditor);
    controller.register_hand(HandType::Database);
    controller.register_hand(HandType::API);
    controller.register_hand(HandType::Network);

    let idle = controller.idle_hands();
    assert_eq!(idle.len(), 7);
}
