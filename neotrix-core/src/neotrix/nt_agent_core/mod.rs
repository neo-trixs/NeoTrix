pub mod agent_kind;
pub mod error;
pub use error::AgentError;

pub mod bus;
pub mod byzantine_consensus;
pub mod consensus;
pub mod lead_agent;
pub mod message;
pub mod orchestrator;
pub mod preview;
pub mod sub_agent;
// ultra_review moved to neotrix/nt_act_code/ultra_review.rs
pub mod agent_memory;
pub mod cdp_session;
pub mod compaction;
pub mod daemon_mode;
pub mod decent_mem;
pub mod design_framework;
pub mod dispatch_pipeline;
pub mod factor_miner;
pub mod harness;
pub mod hyperagent;
pub mod identity;
pub mod permission;
pub mod qr_code;
pub mod quant_data;
pub mod remote_control;
pub mod remote_host;
pub mod task_list;
pub mod tool_result;
pub mod transcript;
pub mod ua_rotation;
pub mod verify_loop;
pub use tool_result::ToolResult;
#[allow(ambiguous_glob_reexports)]
pub use bus::*;
#[allow(ambiguous_glob_reexports)]
pub use byzantine_consensus::*;
#[allow(ambiguous_glob_reexports)]
pub use cdp_session::*;
#[allow(ambiguous_glob_reexports)]
pub use consensus::*;
#[allow(unused_imports)]
pub use decent_mem::*;
#[allow(ambiguous_glob_reexports)]
pub use factor_miner::*;
#[allow(ambiguous_glob_reexports)]
pub use hyperagent::*;
#[allow(ambiguous_glob_reexports)]
pub use identity::*;
#[allow(ambiguous_glob_reexports)]
pub use lead_agent::*;
#[allow(ambiguous_glob_reexports)]
pub use message::*;
#[allow(ambiguous_glob_reexports)]
pub use orchestrator::*;
#[allow(ambiguous_glob_reexports)]
pub use preview::*;
#[allow(ambiguous_glob_reexports)]
pub use quant_data::*;
#[allow(ambiguous_glob_reexports)]
pub use remote_control::*;
#[allow(ambiguous_glob_reexports)]
pub use remote_host::*;
#[allow(ambiguous_glob_reexports)]
pub use sub_agent::*;
#[allow(ambiguous_glob_reexports)]
pub use ua_rotation::*;
// ultra_review moved to neotrix/nt_act_code/ultra_review.rs — re-exported from there
#[allow(ambiguous_glob_reexports)]
pub use agent_memory::*;
#[allow(ambiguous_glob_reexports)]
pub use compaction::*;
pub use daemon_mode::*;
pub use dispatch_pipeline::*;
pub use harness::*;
#[allow(ambiguous_glob_reexports)]
pub use permission::*;
pub use task_list::*;
pub use transcript::*;
pub use verify_loop::*;
pub mod proving_window;
pub use proving_window::*;
pub mod three_role;
pub use three_role::*;
pub mod sandbox_rules;
pub use sandbox_rules::*;

pub mod eval_harness;
pub use eval_harness::*;

pub mod dgmh;
pub use dgmh::{DgmhOrchestrator, EditRecord, EditType, HyperAgentArchive, MetaAgent};

pub mod skill_library;
pub use skill_library::{
    CompositeRecipe, SkillDefinition, SkillLibrary, SkillMatch, SkillStep, SkillType,
};

// pub mod web_agent; // removed — depends on deleted BrowserMCP; use CDPSessionManager directly
// pub mod hierarchical_agent; // removed — depends on deleted web_agent
// pub use hierarchical_agent::CombinedHierarchicalAgent; // removed — module not declared
