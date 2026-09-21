//! Natural Language Chat — single entry point for all user interactions.
//!
//! Replaces 85 specific commands with one conversational interface.

pub mod router;
pub mod response;

pub use response::{ChatResponse, ChatAction};
pub use router::IntentRouter;
