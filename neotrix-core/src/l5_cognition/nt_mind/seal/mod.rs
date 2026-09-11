//! 自进化执行结构图 (Procedural Graph)
//!
//! 基于 arXiv 2609.09153 的程序化图执行框架。
//! 技能节点构成有向图，边表示执行依赖关系，
//! LLM refiner 可根据执行轨迹编辑图拓扑，被拒绝的编辑保留为反模式。

pub mod aegis;
pub mod crystallization;
pub mod distillation;
pub mod harness_evolution;
pub mod harness_optimizer;
pub mod procedural_graph;
