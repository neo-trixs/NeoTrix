mod bridge;
mod client;
mod server;
mod types;

pub use bridge::{a2a_task_to_message, agent_message_to_a2a_task};
pub use client::A2AClient;
pub(crate) use server::emit_event;
pub use server::A2AServer;
pub use types::{
    sign_agent_card, verify_agent_card, A2AArtifact, A2AMessage, A2APart, A2APartType, A2ATask,
    AgentCapabilities, AgentCard, AgentCardSignature, AgentInterface, CancelTaskResponse,
    GetTaskResponse, ProtocolBinding, SendTaskRequest, SendTaskResponse, SkillDecl, TaskEvent,
    TaskState, A2A_PROTOCOL_VERSION,
};
