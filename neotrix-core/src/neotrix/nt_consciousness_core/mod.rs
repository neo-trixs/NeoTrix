//! NeoTrix 意识核心 — 迭代验证引擎
//! 
//! 实现 1000+ 次循环迭代验证，每次寻找漏洞缺陷并循环补齐

pub mod agent;
pub mod probes;
pub mod patches;
pub mod convergence;
pub mod state;
pub mod demo;
pub mod self_observer;
pub mod self_evolver;
pub mod information_absorber;
pub mod knowledge_distiller;
pub mod resource_router;

pub use agent::IterationAgent;
pub use probes::*;
pub use patches::*;
pub use convergence::ConvergenceChecker;
pub use state::StateSnapshot;
pub use self_observer::SelfObserver;
pub use self_evolver::SelfEvolver;
pub use information_absorber::InformationAbsorber;
pub use knowledge_distiller::KnowledgeDistiller;
pub use resource_router::ResourceRouter;
