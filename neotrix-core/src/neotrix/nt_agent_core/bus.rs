use super::byzantine_consensus::{ByzantineConsensus, ConsensusResult, ConsensusStatus};
use super::compaction::{Compactable, CompactionReport, CompactionTier, InfoLossLevel};
use super::error::AgentError;
use super::message::{
    AgentId, AgentMessage, AgentStatus, CoordinationAction, MessageContent, MessagePriority,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;

// ─── Peer-to-Peer Mailbox ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MailboxEntry {
    pub message: AgentMessage,
    pub received_at: Instant,
    pub read: bool,
}

impl MailboxEntry {
    pub fn new(message: AgentMessage) -> Self {
        Self {
            received_at: Instant::now(),
            read: false,
            message,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PeerMailbox {
    mailboxes: HashMap<AgentId, VecDeque<MailboxEntry>>,
    max_per_mailbox: usize,
}

impl PeerMailbox {
    pub fn new(max_per_mailbox: usize) -> Self {
        Self {
            mailboxes: HashMap::new(),
            max_per_mailbox,
        }
    }

    pub fn register(&mut self, agent: AgentId) {
        self.mailboxes
            .entry(agent)
            .or_insert_with(|| VecDeque::with_capacity(self.max_per_mailbox));
    }

    pub fn unregister(&mut self, agent: &AgentId) {
        self.mailboxes.remove(agent);
    }

    pub fn send(&mut self, msg: AgentMessage) -> Result<(), AgentError> {
        for recipient in &msg.recipients {
            let mailbox = self.mailboxes.get_mut(recipient).ok_or_else(|| {
                AgentError::NotFound(format!("Recipient {} has no mailbox", recipient))
            })?;
            if mailbox.len() >= self.max_per_mailbox {
                mailbox.pop_front();
            }
            mailbox.push_back(MailboxEntry::new(msg.clone()));
        }
        Ok(())
    }

    pub fn read_all(&mut self, agent: &AgentId) -> Vec<AgentMessage> {
        let mut result = Vec::new();
        if let Some(mailbox) = self.mailboxes.get_mut(agent) {
            for entry in mailbox.iter_mut() {
                entry.read = true;
                result.push(entry.message.clone());
            }
            mailbox.clear();
        }
        result
    }

    pub fn unread_count(&self, agent: &AgentId) -> usize {
        self.mailboxes
            .get(agent)
            .map(|m| m.iter().filter(|e| !e.read).count())
            .unwrap_or(0)
    }

    pub fn registered_agents(&self) -> HashSet<AgentId> {
        self.mailboxes.keys().cloned().collect()
    }
}

impl Default for PeerMailbox {
    fn default() -> Self {
        Self::new(100)
    }
}

// ─── Forwarding Protocol ──────────────────────────────────────────

/// Forwarding actions an agent can take when it can't handle a message
#[derive(Debug, Clone)]
pub enum ForwardAction {
    /// Forward to a specific agent
    ForwardTo(AgentId),
    /// Broadcast to all agents with a certain capability
    BroadcastToCapability(String),
    /// Escalate to a coordinator/manager
    Escalate(String),
    /// Route by topic hash (for DHT-style routing)
    RouteByTopic(u64),
}

/// Extended message that includes forwarding metadata
#[derive(Debug, Clone)]
pub struct RoutedMessage {
    /// Original sender
    pub from: AgentId,
    /// Original recipient (or name-based target for capability routing)
    pub to: AgentId,
    /// Message content
    pub content: String,
    /// Current forward chain (to detect loops)
    pub forward_chain: Vec<AgentId>,
    /// How many times this has been forwarded
    pub hop_count: u32,
    /// Max hops before dropping
    pub max_hops: u32,
    /// Capability needed to process this
    pub required_capability: Option<String>,
}

impl RoutedMessage {
    pub fn new(from: AgentId, to: AgentId, content: &str) -> Self {
        Self {
            from: from.clone(),
            to,
            content: content.to_string(),
            forward_chain: vec![from],
            hop_count: 0,
            max_hops: 5,
            required_capability: None,
        }
    }
}

/// Agent capability declaration
#[derive(Debug, Clone)]
pub struct AgentCapability {
    pub agent_id: AgentId,
    pub capabilities: Vec<String>,
    pub can_forward: bool,
}

#[derive(Debug, Clone)]
pub struct BusStats {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub messages_expired: u64,
    pub current_queue_size: usize,
    pub registered_agents: usize,
}

impl BusStats {
    pub fn new() -> Self {
        Self {
            messages_sent: 0,
            messages_delivered: 0,
            messages_expired: 0,
            current_queue_size: 0,
            registered_agents: 0,
        }
    }
}

pub struct AgentCommunicationBus {
    agents: HashMap<AgentId, AgentStatus>,
    message_queue: VecDeque<(AgentMessage, Instant)>,
    delivered_messages: VecDeque<AgentMessage>,
    max_queue: usize,
    max_history: usize,
    stats: BusStats,
    /// Peer-to-peer mailbox for direct agent-to-agent messaging
    pub p2p: PeerMailbox,
    /// Capability registry for agent forwarding protocol
    capability_registry: HashMap<AgentId, Vec<String>>,
    /// Optional Byzantine fault-tolerant consensus engine (SAC-based)
    consensus: Option<ByzantineConsensus>,
}

impl AgentCommunicationBus {
    pub fn new(max_queue: usize) -> Self {
        Self {
            agents: HashMap::new(),
            message_queue: VecDeque::with_capacity(max_queue),
            delivered_messages: VecDeque::new(),
            max_queue: max_queue.max(1),
            max_history: 1000,
            stats: BusStats::new(),
            p2p: PeerMailbox::new(100),
            capability_registry: HashMap::new(),
            consensus: None,
        }
    }

    pub fn register_agent(&mut self, id: AgentId, status: AgentStatus) -> Result<(), AgentError> {
        if self.agents.contains_key(&id) {
            return Err(AgentError::InvalidState(format!(
                "Agent {} already registered",
                id
            )));
        }
        self.agents.insert(id.clone(), status);
        self.p2p.register(id);
        self.stats.registered_agents = self.agents.len();
        Ok(())
    }

    pub fn unregister_agent(&mut self, id: &AgentId) {
        self.agents.remove(id);
        self.p2p.unregister(id);
        self.stats.registered_agents = self.agents.len();
    }

    pub fn send(&mut self, message: AgentMessage) -> Result<(), AgentError> {
        if !self.agents.contains_key(&message.sender) {
            return Err(AgentError::NotFound(format!(
                "Sender {} not registered",
                message.sender
            )));
        }

        if !message.is_broadcast() {
            for r in &message.recipients {
                if !self.agents.contains_key(r) {
                    return Err(AgentError::NotFound(format!(
                        "Recipient {} not registered",
                        r
                    )));
                }
            }
        }

        if self.message_queue.len() >= self.max_queue {
            return Err(AgentError::CommunicationFailed(
                "Message queue is full".into(),
            ));
        }

        self.stats.messages_sent += 1;
        self.stats.current_queue_size = self.message_queue.len() + 1;
        self.message_queue.push_back((message, Instant::now()));
        Ok(())
    }

    pub fn broadcast(
        &mut self,
        sender: &AgentId,
        content: MessageContent,
    ) -> Result<(), AgentError> {
        if !self.agents.contains_key(sender) {
            return Err(AgentError::NotFound(format!(
                "Sender {} not registered",
                sender
            )));
        }
        let msg = AgentMessage::new(
            sender.clone(),
            vec![],
            content,
            MessagePriority::Normal,
            std::time::Duration::from_secs(60),
        );
        self.send(msg)
    }

    pub fn deliver(&mut self) -> Vec<AgentMessage> {
        let mut delivered = Vec::new();
        let mut remaining = VecDeque::new();

        while let Some((msg, enqueued)) = self.message_queue.pop_front() {
            if msg.expired() {
                self.stats.messages_expired += 1;
                continue;
            }

            let recipient_set: Vec<AgentId> = if msg.is_broadcast() {
                self.agents
                    .keys()
                    .filter(|a| *a != &msg.sender)
                    .cloned()
                    .collect()
            } else {
                msg.recipients
                    .iter()
                    .filter(|r| self.agents.contains_key(r))
                    .cloned()
                    .collect()
            };

            if recipient_set.is_empty() {
                remaining.push_back((msg, enqueued));
                continue;
            }

            let mut enriched = msg;
            enriched.recipients = recipient_set;
            self.stats.messages_delivered += 1;
            delivered.push(enriched);
        }

        self.message_queue = remaining;
        self.stats.current_queue_size = self.message_queue.len();

        for m in &delivered {
            self.delivered_messages.push_back(m.clone());
        }
        while self.delivered_messages.len() > self.max_history {
            self.delivered_messages.pop_front();
        }

        delivered
    }

    pub fn deliver_prioritized(&mut self) -> Vec<AgentMessage> {
        let mut messages: Vec<(AgentMessage, Instant)> = self.message_queue.drain(..).collect();
        messages.sort_by(|(a, _), (b, _)| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| b.conversation_id.cmp(&a.conversation_id))
        });

        self.message_queue = messages.into();
        self.deliver()
    }

    pub fn query_status(&self) -> &BusStats {
        &self.stats
    }

    pub fn agent_status(&self, id: &AgentId) -> Option<AgentStatus> {
        self.agents.get(id).copied()
    }

    pub fn update_status(&mut self, id: &AgentId, status: AgentStatus) {
        if let Some(s) = self.agents.get_mut(id) {
            *s = status;
        }
    }

    pub fn pending_count(&self) -> usize {
        self.message_queue.len()
    }

    pub fn registered_agents(&self) -> impl Iterator<Item = (&AgentId, &AgentStatus)> {
        self.agents.iter()
    }

    pub fn is_registered(&self, id: &AgentId) -> bool {
        self.agents.contains_key(id)
    }

    pub fn clear_expired(&mut self) {
        let before = self.message_queue.len();
        self.message_queue.retain(|(msg, _)| !msg.expired());
        self.stats.messages_expired += (before - self.message_queue.len()) as u64;
        self.stats.current_queue_size = self.message_queue.len();
    }

    pub fn set_history_limit(&mut self, max: usize) {
        self.max_history = max.max(10);
        while self.delivered_messages.len() > self.max_history {
            self.delivered_messages.pop_front();
        }
    }

    /// Send a direct peer-to-peer message to another agent's mailbox.
    /// Unlike send(), this does not go through the main message queue.
    pub fn p2p_send(
        &mut self,
        sender: &AgentId,
        recipient: AgentId,
        content: MessageContent,
    ) -> Result<(), AgentError> {
        if !self.agents.contains_key(sender) {
            return Err(AgentError::NotFound(format!(
                "Sender {} not registered",
                sender
            )));
        }
        let msg = AgentMessage::new(
            sender.clone(),
            vec![recipient],
            content,
            MessagePriority::Normal,
            std::time::Duration::from_secs(60),
        );
        self.p2p.send(msg)
    }

    // ─── Forwarding Protocol ──────────────────────────────────────

    /// Register an agent with its capabilities for message forwarding.
    pub fn register_capability(&mut self, agent_id: AgentId, capabilities: Vec<String>) {
        self.capability_registry.insert(agent_id, capabilities);
    }

    /// Remove an agent's capability registration.
    pub fn unregister_capability(&mut self, agent_id: &AgentId) {
        self.capability_registry.remove(agent_id);
    }

    /// Forward a routed message: find a capable recipient and deliver.
    /// Returns Ok(()) if the message was forwarded, Err with reason otherwise.
    pub fn forward_message(&mut self, msg: RoutedMessage) -> Result<(), String> {
        if msg.hop_count >= msg.max_hops {
            return Err("MAX_HOPS_EXCEEDED".into());
        }

        let mut next = msg;
        next.hop_count += 1;

        // Determine candidate recipients
        let candidates: Vec<AgentId> = if let Some(ref cap) = next.required_capability {
            // Find agents that have the required capability
            self.capability_registry
                .iter()
                .filter(|(id, caps)| {
                    caps.contains(cap)
                        && !next.forward_chain.contains(id)
                        && self.agents.contains_key(id)
                        && self
                            .agents
                            .get(id)
                            .map(|s| s.is_available())
                            .unwrap_or(false)
                })
                .map(|(id, _)| id.clone())
                .collect()
        } else {
            // If no specific capability, try the original recipient
            if self.agents.contains_key(&next.to) && !next.forward_chain.contains(&next.to) {
                vec![next.to.clone()]
            } else {
                return Err("NO_VALID_RECIPIENT".into());
            }
        };

        if candidates.is_empty() {
            return Err("NO_CAPABLE_AGENT".into());
        }

        let target = candidates[0].clone();
        next.forward_chain.push(target.clone());

        // Deliver via the existing bus as a Coordination::Delegate message
        let agent_msg = AgentMessage::new(
            next.from.clone(),
            vec![target],
            MessageContent::Coordination {
                action: CoordinationAction::Delegate,
                rationale: next.content.clone(),
            },
            MessagePriority::Normal,
            std::time::Duration::from_secs(60),
        );

        self.send(agent_msg).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Escalate a message to a coordinator-level agent.
    /// Wraps content as `[ESCALATE: <reason>] <content>` and forwards
    /// with required_capability="coordinator".
    pub fn escalate_message(
        &mut self,
        from: AgentId,
        content: &str,
        reason: &str,
    ) -> Result<(), String> {
        let wrapped = format!("[ESCALATE: {}] {}", reason, content);
        // Use a sentinel agent ID for routing; coordinator lookup is by capability
        let coord_id = AgentId::new("coordinator", "0");
        let msg = RoutedMessage {
            from,
            to: coord_id,
            content: wrapped,
            forward_chain: Vec::new(),
            hop_count: 0,
            max_hops: 5,
            required_capability: Some("coordinator".into()),
        };
        self.forward_message(msg)
    }

    /// Return all (agent_id, capabilities) pairs registered in the capability index.
    pub fn capability_index(&self) -> Vec<(AgentId, Vec<String>)> {
        self.capability_registry
            .iter()
            .map(|(id, caps)| (id.clone(), caps.clone()))
            .collect()
    }

    // ─── Byzantine Consensus ─────────────────────────────────────

    /// Enable Byzantine fault-tolerant consensus with the given fault tolerance.
    ///
    /// `max_faulty` = maximum number of Byzantine (malicious/faulty) agents to tolerate.
    /// Quorum size will be 3f+1 automatically.
    pub fn with_consensus(mut self, max_faulty: usize) -> Self {
        self.consensus = Some(ByzantineConsensus::new(max_faulty));
        self
    }

    /// Deliver a message and run BFT consensus on agent responses.
    ///
    /// 1. Sends the message as a broadcast to all registered agents
    /// 2. Delivers via the existing message queue
    /// 3. Collects response messages whose `reply_to` matches this message's ID
    /// 4. Evaluates consensus using the SAC MSR algorithm
    ///
    /// If consensus engine is not configured (via `with_consensus`), returns
    /// `ConsensusStatus::Confirmed` immediately (pass-through mode).
    pub fn deliver_with_consensus(&mut self, message: &AgentMessage) -> ConsensusResult {
        // Take ownership of consensus temporarily to avoid conflicting mutable borrows on self
        let mut engine_opt = self.consensus.take();

        let result = match engine_opt.as_mut() {
            None => ConsensusResult {
                message_id: message.id.to_string(),
                status: ConsensusStatus::Confirmed,
                confirmations: 1,
                total_voters: 1,
                threshold: 1,
            },
            Some(engine) => {
                if let Err(e) = self.send(message.clone()) {
                    log::warn!("[bus] deliver_with_consensus send failed: {e}");
                    ConsensusResult {
                        message_id: message.id.to_string(),
                        status: ConsensusStatus::Conflict,
                        confirmations: 0,
                        total_voters: 0,
                        threshold: engine.quorum_size(),
                    }
                } else {
                    self.deliver();

                    let responses: Vec<(AgentId, String)> = self
                        .delivered_messages
                        .iter()
                        .filter(|m| m.reply_to == Some(message.id))
                        .filter_map(|m| match &m.content {
                            MessageContent::Response { answer, .. } => {
                                Some((m.sender.clone(), answer.clone()))
                            }
                            MessageContent::TaskResult { output, .. } => {
                                Some((m.sender.clone(), output.clone()))
                            }
                            _ => None,
                        })
                        .collect();

                    engine.evaluate_consensus(message.id, &responses)
                }
            }
        };

        // Restore consensus engine (may have been mutated by evaluate_consensus)
        self.consensus = engine_opt;
        result
    }
}

impl Compactable for AgentCommunicationBus {
    fn estimated_bytes(&self) -> u64 {
        let queue_bytes: usize = self
            .message_queue
            .iter()
            .map(|(msg, _)| std::mem::size_of_val(msg) + msg.recipients.len() * 32)
            .sum();
        let history_bytes: usize = self
            .delivered_messages
            .iter()
            .map(|m| std::mem::size_of_val(m))
            .sum();
        let agents_bytes = self.agents.len() * 64;
        (queue_bytes + history_bytes + agents_bytes) as u64
    }

    fn compact(&mut self, tier: CompactionTier) -> CompactionReport {
        match tier {
            CompactionTier::Snip => {
                let before = self.message_queue.len();
                let keep = self.message_queue.split_off(before / 2);
                let removed = before - keep.len();
                self.message_queue = keep;
                CompactionReport {
                    tier,
                    bytes_freed: (removed * 256) as u64,
                    info_loss: InfoLossLevel::High,
                    items_removed: removed,
                }
            }
            CompactionTier::Microcompact => {
                let before = self.message_queue.len();
                self.message_queue
                    .retain(|(msg, _)| msg.priority.rank() >= MessagePriority::Normal.rank());
                let removed = before - self.message_queue.len();
                CompactionReport {
                    tier,
                    bytes_freed: (removed * 256) as u64,
                    info_loss: InfoLossLevel::Medium,
                    items_removed: removed,
                }
            }
            CompactionTier::Collapse => {
                let before = self.delivered_messages.len();
                self.delivered_messages.truncate(self.max_history / 2);
                self.set_history_limit(self.max_history / 2);
                CompactionReport {
                    tier,
                    bytes_freed: (before.saturating_sub(self.delivered_messages.len()) * 256)
                        as u64,
                    info_loss: InfoLossLevel::Low,
                    items_removed: before - self.delivered_messages.len(),
                }
            }
            CompactionTier::AutoCompact => {
                let before = self.message_queue.len();
                let summary = if let Some((msg, _)) = self.message_queue.back() {
                    Some(msg.clone())
                } else {
                    None
                };
                self.message_queue.clear();
                if let Some(mut m) = summary {
                    m.content = MessageContent::StatusUpdate {
                        status: AgentStatus::Idle,
                        progress: 1.0,
                        message: "auto-compacted:bus".into(),
                    };
                    self.message_queue.push_back((m, Instant::now()));
                }
                CompactionReport {
                    tier,
                    bytes_freed: (before * 256) as u64,
                    info_loss: InfoLossLevel::Medium,
                    items_removed: before.saturating_sub(1),
                }
            }
            CompactionTier::Blocking => CompactionReport {
                tier,
                bytes_freed: 0,
                info_loss: InfoLossLevel::None,
                items_removed: 0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::time::Duration;

    fn test_agent(name: &str) -> AgentId {
        AgentId::with_random_instance(name, "1.0")
    }

    #[serial]
    #[test]
    fn test_register_and_unregister() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("alpha");
        assert!(bus.register_agent(a.clone(), AgentStatus::Idle).is_ok());
        assert_eq!(bus.query_status().registered_agents, 1);
        assert!(bus.register_agent(a.clone(), AgentStatus::Busy).is_err());

        bus.unregister_agent(&a);
        assert_eq!(bus.query_status().registered_agents, 0);
    }

    #[test]
    fn test_send_and_deliver() {
        let mut bus = AgentCommunicationBus::new(100);
        let alice = test_agent("alice");
        let bob = test_agent("bob");
        let _ = bus.register_agent(alice.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(bob.clone(), AgentStatus::Idle);

        let msg = AgentMessage::new(
            alice.clone(),
            vec![bob.clone()],
            MessageContent::Query {
                question: "hello?".into(),
                context: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(60),
        );
        assert!(bus.send(msg).is_ok());

        let delivered = bus.deliver();
        assert_eq!(delivered.len(), 1);
        assert_eq!(bus.query_status().messages_delivered, 1);
        assert_eq!(bus.query_status().current_queue_size, 0);
    }

    #[test]
    fn test_send_unregistered_sender() {
        let mut bus = AgentCommunicationBus::new(100);
        let rogue = test_agent("rogue");
        let bob = test_agent("bob");
        let _ = bus.register_agent(bob.clone(), AgentStatus::Idle);

        let msg = AgentMessage::new(
            rogue,
            vec![bob],
            MessageContent::Response {
                answer: "hi".into(),
                sources: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(10),
        );
        assert!(bus.send(msg).is_err());
    }

    #[test]
    fn test_send_unregistered_recipient() {
        let mut bus = AgentCommunicationBus::new(100);
        let alice = test_agent("alice");
        let phantom = test_agent("phantom");
        let _ = bus.register_agent(alice.clone(), AgentStatus::Idle);

        let msg = AgentMessage::new(
            alice,
            vec![phantom],
            MessageContent::Response {
                answer: "hi".into(),
                sources: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(10),
        );
        assert!(bus.send(msg).is_err());
    }

    #[test]
    fn test_broadcast() {
        let mut bus = AgentCommunicationBus::new(100);
        let alice = test_agent("alice");
        let bob = test_agent("bob");
        let charlie = test_agent("charlie");

        let _ = bus.register_agent(alice.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(bob.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(charlie.clone(), AgentStatus::Idle);

        assert!(bus
            .broadcast(
                &alice,
                MessageContent::StatusUpdate {
                    status: AgentStatus::Busy,
                    progress: 0.5,
                    message: "working".into(),
                }
            )
            .is_ok());

        let delivered = bus.deliver();
        // bob and charlie (not alice)
        assert_eq!(delivered.len(), 1);
    }

    #[test]
    fn test_expired_message_skipped() {
        let mut bus = AgentCommunicationBus::new(100);
        let alice = test_agent("alice");
        let bob = test_agent("bob");
        let _ = bus.register_agent(alice.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(bob.clone(), AgentStatus::Idle);

        let mut msg = AgentMessage::new(
            alice.clone(),
            vec![bob.clone()],
            MessageContent::Query {
                question: "?".into(),
                context: vec![],
            },
            MessagePriority::Low,
            Duration::from_nanos(1),
        );
        msg.timestamp = Instant::now() - Duration::from_secs(10);
        assert!(bus.send(msg).is_ok());

        let delivered = bus.deliver();
        assert_eq!(delivered.len(), 0);
        assert_eq!(bus.query_status().messages_expired, 1);
    }

    #[test]
    fn test_queue_full() {
        let mut bus = AgentCommunicationBus::new(3);
        let a = test_agent("a");
        let b = test_agent("b");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(b.clone(), AgentStatus::Idle);

        for i in 0..3 {
            let msg = AgentMessage::new(
                a.clone(),
                vec![b.clone()],
                MessageContent::Query {
                    question: format!("q{}", i),
                    context: vec![],
                },
                MessagePriority::Normal,
                Duration::from_secs(60),
            );
            assert!(bus.send(msg).is_ok());
        }

        let overflow = AgentMessage::new(
            a,
            vec![b],
            MessageContent::Query {
                question: "overflow".into(),
                context: vec![],
            },
            MessagePriority::Normal,
            Duration::from_secs(60),
        );
        assert!(bus.send(overflow).is_err());
    }

    #[test]
    fn test_agent_status_lifecycle() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("agent_x");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        assert_eq!(bus.agent_status(&a), Some(AgentStatus::Idle));

        bus.update_status(&a, AgentStatus::Busy);
        assert_eq!(bus.agent_status(&a), Some(AgentStatus::Busy));

        bus.unregister_agent(&a);
        assert_eq!(bus.agent_status(&a), None);
    }

    #[test]
    fn test_deliver_prioritized() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("a");
        let b = test_agent("b");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(b.clone(), AgentStatus::Idle);

        let low = AgentMessage::new(
            a.clone(),
            vec![b.clone()],
            MessageContent::Query {
                question: "low".into(),
                context: vec![],
            },
            MessagePriority::Low,
            Duration::from_secs(60),
        );
        let high = AgentMessage::new(
            a.clone(),
            vec![b.clone()],
            MessageContent::Query {
                question: "high".into(),
                context: vec![],
            },
            MessagePriority::High,
            Duration::from_secs(60),
        );
        let crit = AgentMessage::new(
            a,
            vec![b],
            MessageContent::Query {
                question: "crit".into(),
                context: vec![],
            },
            MessagePriority::Critical,
            Duration::from_secs(60),
        );

        if let Err(e) = bus.send(low) {
            log::warn!("[bus] send failed: {}", e);
        }
        if let Err(e) = bus.send(high) {
            log::warn!("[bus] send failed: {}", e);
        }
        if let Err(e) = bus.send(crit) {
            log::warn!("[bus] send failed: {}", e);
        }

        let delivered = bus.deliver_prioritized();
        assert_eq!(delivered.len(), 3);
    }

    #[test]
    fn test_clear_expired() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("a");
        let b = test_agent("b");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(b.clone(), AgentStatus::Idle);

        let mut old = AgentMessage::new(
            a.clone(),
            vec![b.clone()],
            MessageContent::StatusUpdate {
                status: AgentStatus::Idle,
                progress: 1.0,
                message: "old".into(),
            },
            MessagePriority::Low,
            Duration::from_nanos(1),
        );
        old.timestamp = Instant::now() - Duration::from_secs(10);
        if let Err(e) = bus.send(old) {
            log::warn!("[bus] send failed: {}", e);
        }

        let fresh = AgentMessage::new(
            a,
            vec![b],
            MessageContent::StatusUpdate {
                status: AgentStatus::Busy,
                progress: 0.0,
                message: "fresh".into(),
            },
            MessagePriority::Normal,
            Duration::from_secs(60),
        );
        if let Err(e) = bus.send(fresh) {
            log::warn!("[bus] send failed: {}", e);
        }

        assert_eq!(bus.pending_count(), 2);
        bus.clear_expired();
        assert_eq!(bus.pending_count(), 1);
    }

    #[test]
    fn test_is_registered() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("exists");
        let b = test_agent("ghost");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        assert!(bus.is_registered(&a));
        assert!(!bus.is_registered(&b));
    }

    #[test]
    fn test_pending_count() {
        let mut bus = AgentCommunicationBus::new(100);
        let a = test_agent("a");
        let b = test_agent("b");
        let _ = bus.register_agent(a.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(b.clone(), AgentStatus::Idle);

        assert_eq!(bus.pending_count(), 0);
        for _ in 0..5 {
            let msg = AgentMessage::new(
                a.clone(),
                vec![b.clone()],
                MessageContent::Query {
                    question: "q".into(),
                    context: vec![],
                },
                MessagePriority::Normal,
                Duration::from_secs(60),
            );
            if let Err(e) = bus.send(msg) {
                log::warn!("[bus::test] send failed: {e}");
            }
        }
        assert_eq!(bus.pending_count(), 5);
    }

    #[test]
    fn test_register_capability_and_index() {
        let mut bus = AgentCommunicationBus::new(100);
        let alpha = test_agent("alpha");
        let beta = test_agent("beta");

        let _ = bus.register_agent(alpha.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(beta.clone(), AgentStatus::Idle);

        bus.register_capability(alpha.clone(), vec!["search".into(), "summarize".into()]);
        bus.register_capability(beta.clone(), vec!["coordinator".into()]);

        let index = bus.capability_index();
        assert_eq!(index.len(), 2);

        let alpha_entry = index.iter().find(|(id, _)| id.name == "alpha").unwrap();
        assert!(alpha_entry.1.contains(&"search".to_string()));
        assert!(alpha_entry.1.contains(&"summarize".to_string()));

        let beta_entry = index.iter().find(|(id, _)| id.name == "beta").unwrap();
        assert!(beta_entry.1.contains(&"coordinator".to_string()));
    }

    #[test]
    fn test_forward_message_to_capable_agent() {
        let mut bus = AgentCommunicationBus::new(100);
        let sender = test_agent("sender");
        let worker = test_agent("worker");

        let _ = bus.register_agent(sender.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(worker.clone(), AgentStatus::Idle);
        bus.register_capability(worker.clone(), vec!["search".into()]);

        let msg = RoutedMessage {
            from: sender.clone(),
            to: worker.clone(),
            content: "find me results".into(),
            forward_chain: vec![sender.clone()],
            hop_count: 0,
            max_hops: 5,
            required_capability: Some("search".into()),
        };

        assert!(bus.forward_message(msg).is_ok());
    }

    #[test]
    fn test_forward_message_exceeds_max_hops() {
        let mut bus = AgentCommunicationBus::new(100);
        let sender = test_agent("sender");

        let msg = RoutedMessage {
            from: sender.clone(),
            to: AgentId::new("nowhere", "0"),
            content: "too far".into(),
            forward_chain: vec![sender],
            hop_count: 5,
            max_hops: 5,
            required_capability: None,
        };

        let result = bus.forward_message(msg);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "MAX_HOPS_EXCEEDED");
    }

    #[test]
    fn test_escalate_wraps_content() {
        let mut bus = AgentCommunicationBus::new(100);
        let agent = test_agent("agent");
        let coordinator = test_agent("coordinator");

        let _ = bus.register_agent(agent.clone(), AgentStatus::Idle);
        let _ = bus.register_agent(coordinator.clone(), AgentStatus::Idle);
        bus.register_capability(coordinator, vec!["coordinator".into()]);

        let result = bus.escalate_message(agent, "something broke", "hardware_failure");
        assert!(result.is_ok());
    }
}
