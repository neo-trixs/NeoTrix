use serde::{Serialize, Deserialize};
use crate::neotrix::orchestrator::task_spec::TaskSpec;
use crate::neotrix::orchestrator::state_graph::{StateGraph, ArtifactNode, ArtifactType};

/// Types of orchestration processes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProcessType {
    /// Tasks executed in linear order.
    Sequential,
    /// Manager agent delegates to specialized sub-agents.
    Hierarchical,
    /// Custom DAG defined by the caller.
    CustomDag,
}

/// Trait for process execution strategies.
pub trait ProcessDefinition: std::fmt::Debug + Send + Sync {
    fn process_type(&self) -> ProcessType;
    fn build_graph(&self, tasks: &[TaskSpec]) -> StateGraph;
    fn describe(&self) -> &'static str;
}

/// Sequential process — executes tasks in linear order.
#[derive(Debug, Clone)]
pub struct SequentialProcess;

impl ProcessDefinition for SequentialProcess {
    fn process_type(&self) -> ProcessType {
        ProcessType::Sequential
    }

    fn build_graph(&self, tasks: &[TaskSpec]) -> StateGraph {
        let mut graph = StateGraph::new();
        for (i, task) in tasks.iter().enumerate() {
            graph.add_node(ArtifactNode::new(&task.id, ArtifactType::Task, &task.description));
            if i > 0 {
                graph.add_edge(&tasks[i - 1].id, &task.id);
            }
        }
        graph
    }

    fn describe(&self) -> &'static str {
        "Sequential: tasks executed one after another"
    }
}

/// Hierarchical process — manager delegates to specialized agents.
#[derive(Debug, Clone)]
pub struct HierarchicalProcess;

impl ProcessDefinition for HierarchicalProcess {
    fn process_type(&self) -> ProcessType {
        ProcessType::Hierarchical
    }

    fn build_graph(&self, tasks: &[TaskSpec]) -> StateGraph {
        let mut graph = StateGraph::new();
        let manager_id = "manager".to_string();
        graph.add_node(ArtifactNode::new(&manager_id, ArtifactType::Proposal, "Manager agent"));
        for task in tasks.iter() {
            graph.add_node(ArtifactNode::new(&task.id, ArtifactType::Task, &task.description));
            graph.add_edge(&manager_id, &task.id);
        }
        graph
    }

    fn describe(&self) -> &'static str {
        "Hierarchical: manager delegates tasks, collects results"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_process() {
        let process = SequentialProcess;
        assert_eq!(process.process_type(), ProcessType::Sequential);
    }

    #[test]
    fn test_hierarchical_process() {
        let process = HierarchicalProcess;
        assert_eq!(process.process_type(), ProcessType::Hierarchical);
    }

    #[test]
    fn test_sequential_graph_build() {
        let tasks = vec![
            TaskSpec::new("Design API"),
            TaskSpec::new("Implement API"),
            TaskSpec::new("Test API"),
        ];
        let process = SequentialProcess;
        let graph = process.build_graph(&tasks);
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn test_hierarchical_graph_build() {
        let tasks = vec![
            TaskSpec::new("Research"),
            TaskSpec::new("Implement"),
            TaskSpec::new("Review"),
        ];
        let process = HierarchicalProcess;
        let graph = process.build_graph(&tasks);
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
    }

    #[test]
    fn test_process_descriptions() {
        assert!(SequentialProcess.describe().contains("Sequential"));
        assert!(HierarchicalProcess.describe().contains("Hierarchical"));
    }
}
