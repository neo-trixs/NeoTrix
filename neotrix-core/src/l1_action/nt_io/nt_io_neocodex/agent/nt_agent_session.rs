// ── NeoCodex agent session: tags / rename / brain / mode / resume (split from agent.rs, behavior unchanged) ──

use std::sync::Arc;
use std::time::Instant;

use super::NeoCodexAgent;
use super::super::hooks::{HookResult, ToolCallContext};
use super::super::provider::NeoCodexMode;
use super::super::wire::{WireEvent, WireSession};

impl NeoCodexAgent {
    /// Register a pre-tool lifecycle hook (from Kimi Code lifecycle hooks)
    pub(crate) fn _add_pre_hook<F>(&mut self, name: &str, hook: F)
    where
        F: Fn(ToolCallContext) -> HookResult + Send + Sync + 'static,
    {
        self.hooks.register_pre(name, Arc::new(hook));
    }

    pub(crate) fn _set_consciousness_tree(
        &mut self,
        tree: crate::l5_cognition::nt_core_consciousness_tree::ConsciousnessTree,
    ) {
        self.consciousness = Some(tree);
    }

    pub(crate) fn _set_event_bus(&mut self, bus: crate::neotrix::nt_core_event_bus::EventBus) {
        self.event_bus = Some(bus);
    }

    pub fn set_brain(
        &mut self,
        brain: Arc<
            tokio::sync::RwLock<dyn crate::l0_substrate::nt_core_traits::BrainHandle>,
        >,
    ) {
        self.brain = Some(brain);
    }

    /// Set an explicit mode (Agent/Shell/Plan)
    pub fn set_mode(&mut self, mode: NeoCodexMode) {
        if self.state.mode == mode {
            return;
        }
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange { from, to: mode });
        self.state.mode = mode;
        self.state.mode_start = Instant::now();
    }

    /// Persist a user-chosen session name into the wire stream (overrides
    /// the derived first-message name on read).
    pub fn rename_session(&mut self, name: &str) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        // 保留既有标签（重命名不动 tags；读旧态标签并回写）
        let tags = self.read_session_tags();
        self.wire.record(WireEvent::SessionMeta {
            name: name.trim().to_string(),
            timestamp: ts,
            tags,
        });
    }

    /// Read the persisted tag set for the current wire session (from last
    /// SessionMeta). Empty when none recorded yet.
    fn read_session_tags(&self) -> Vec<String> {
        self.wire
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }))
            .and_then(|e| match e {
                WireEvent::SessionMeta { tags, .. } => Some(tags.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    /// Persist a tag onto the current session (deduped). Writes a new
    /// SessionMeta carrying the merged tag set.
    pub(crate) fn _tag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let clean = tag.trim().to_lowercase().replace(' ', "-");
        if clean.is_empty() || tags.contains(&clean) {
            return;
        }
        tags.push(clean);
        self.write_session_tags(tags);
    }

    /// Remove a tag from the current session. No-op when absent.
    pub(crate) fn _untag_session(&mut self, tag: &str) {
        let mut tags = self.read_session_tags();
        let before = tags.len();
        tags.retain(|t| t != tag);
        if tags.len() != before {
            self.write_session_tags(tags);
        }
    }

    /// Overwrite persisted tags (rename/tag/untag 共用落盘点)。
    fn write_session_tags(&mut self, tags: Vec<String>) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let meta = self
            .wire
            .events
            .iter()
            .rev()
            .find(|e| matches!(e, WireEvent::SessionMeta { .. }));
        let name = meta
            .and_then(|e| match e {
                WireEvent::SessionMeta { name, .. } => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| self.wire.session_id.clone());
        self.wire.record(WireEvent::SessionMeta {
            name,
            timestamp: ts,
            tags,
        });
    }

    /// Detach the agent from its current wire file (P2-3). After a session is
    /// archived/deleted while active, the stale `wire.path` would otherwise
    /// recreate the file on the next `record()` (`create+append`) and split
    /// the conversation into a divergent duplicate. Resets to a fresh empty
    /// session so the next turn starts clean.
    pub(crate) fn _detach_wire(&mut self) {
        self.wire = WireSession::new(&format!(
            "s-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.turn_count = 0;
        self.state.tool_call_count = 0;
    }

    /// Toggle between Agent and Shell mode (from Kimi Code Ctrl-X)
    pub fn toggle_mode(&mut self) -> NeoCodexMode {
        let from = self.state.mode;
        self.state.mode = match from {
            NeoCodexMode::Agent => NeoCodexMode::Shell,
            NeoCodexMode::Shell => NeoCodexMode::Agent,
            NeoCodexMode::Plan => NeoCodexMode::Agent,
        };
        self.state.mode_start = Instant::now();
        self.wire.record(WireEvent::ModeChange {
            from,
            to: self.state.mode,
        });
        self.state.mode
    }

    /// Switch to Plan mode (from Kimi Code Shift-Tab / Claude Code Plan)
    pub fn set_plan_mode(&mut self) {
        let from = self.state.mode;
        self.wire.record(WireEvent::ModeChange {
            from,
            to: NeoCodexMode::Plan,
        });
        self.state.mode = NeoCodexMode::Plan;
        self.state.mode_start = Instant::now();
    }

    /// Set the streaming-path permission policy (P0-2). Called by the desktop
    /// command layer from the UI's permission_mode (auto/manual/accept_edits/plan).
    pub(crate) fn _set_permission_mode(&mut self, mode: &str) {
        self.state.permission_mode = mode.to_string();
        if mode == "plan" {
            self.set_plan_mode();
        }
    }

    /// Restore prior session events from the wire file into the context
    /// pipeline (G2 session continuity, matching Claude Code `--resume`).
    /// Returns the number of events restored.
    /// Clear in-memory context and re-restore it from the wire file. Used after
    /// an edit/delete/regenerate rewrites the JSONL so the agent's next turn is
    /// built from the corrected history, not stale in-memory state.
    pub(crate) fn _rebuild_context_from_wire(&mut self) -> usize {
        self.context.turns.clear();
        self.state.tokens_used = 0;
        self.state.tool_call_count = 0;
        self.resume_session()
    }

    pub fn resume_session(&mut self) -> usize {
        let events = self.wire.load();
        let mut restored = 0;
        for event in events {
            match event {
                WireEvent::UserMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("user", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::AgentMessage { content, .. } => {
                    let est = content.len() / 4;
                    self.context.push("assistant", content, est);
                    self.state.tokens_used += est;
                    restored += 1;
                }
                WireEvent::ToolCall {
                    name, args, result, ..
                } => {
                    self.context.push(
                        "user",
                        format!("[tool {}] args={} result={}", name, args, result),
                        (name.len() + args.len() + result.len()) / 4,
                    );
                    self.state.tool_call_count += 1;
                    restored += 1;
                }
                WireEvent::ModeChange { to, .. } => {
                    self.state.mode = to;
                    restored += 1;
                }
                _ => {}
            }
        }
        restored
    }
}
