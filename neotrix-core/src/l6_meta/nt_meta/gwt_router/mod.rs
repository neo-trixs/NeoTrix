//! GWT Router — Global Workspace Theory attention routing for NeoTrix.
//!
//! Implements cost-aware attention routing (Axiom A1): not all tasks need the strongest model.
//! Components:
//! - `config` — GwtConfig: budget, salience weights, cost weight factor
//! - `salience` — SalienceCalculator: compute composite salience scores
//! - `broadcast` — GlobalWorkspace: broadcast content to relevant subscribers
//! - `attention` — AttentionManager: allocate attention budget across components
//! - `cost_weight` — CostWeightedRouting: route tasks to cheapest capable model

pub mod attention;
pub mod broadcast;
pub mod config;
pub mod cost_weight;
pub mod salience;

pub use attention::{Allocation, AttentionManager};
pub use broadcast::{BroadcastContent, BroadcastPriority, DeliveryReason, DeliveryRecord, GlobalWorkspace, Subscriber};
pub use config::{GwtConfig, SalienceWeights};
pub use cost_weight::{CostWeightConfig, CostWeightedRouting, ModelTier, RoutingDecision, RoutingReason};
pub use salience::{Context, SalienceCalculator, SalienceComponents, SalienceScore, Task};
