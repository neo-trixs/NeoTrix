pub mod graph_memory;
pub mod memory_stream;
pub mod sim_agent;
pub mod spatial_memory;
pub mod action_awareness;
pub mod planning;
pub mod event_reactive;
pub mod personality_drift;
pub mod reflection;
pub mod action_costs;
pub mod pheromone;
pub mod behavior_tree;
pub mod stimulus;
pub mod decision_engine;
pub mod emotional_bias;

// NT-MEMORY domain: knowledge representation and retrieval
pub mod semantic_memory;
pub mod memory_consolidation;
pub mod memory_retrieval;
pub mod adversarial_memory;
pub mod causal_memory;
pub mod knowledge_graph;
pub mod memory_manager;

// Fusion Adapters (External Model Integration)
pub mod goal_outcome_feedback;
pub mod intention_commitment;
pub mod thought_generation;
pub mod social_learning;

pub use sim_agent::*;
pub use action_awareness::*;
pub use planning::*;
pub use event_reactive::*;
pub use personality_drift::*;
pub use action_costs::*;
pub use pheromone::*;
pub use memory_manager::MemoryManager;
pub use memory_manager::MemoryStats;
pub use decision_engine::DecisionEngine;
