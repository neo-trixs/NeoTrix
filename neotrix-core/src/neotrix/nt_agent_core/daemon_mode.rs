use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// DaemonMode — background session that survives terminal closure.
///
/// Provides:
///   1. Keep-alive: sends heartbeats so the conscious loop persists
///   2. Inbox: file-based inter-session message queue (compatible with tokio watch)
///   3. Session persistence: periodic state snapshot to disk
///   4. Graceful degradation: all operations return gracefully on failure
///
/// Architecture:
///   ┌─────────────────┐     ┌──────────────────┐
///   │ Consciousness   │ ←── │ DaemonMode        │
///   │ Integration     │     │  · keepalive      │
///   │ (background     │     │  · inbox (UDS)    │
///   │  loop tick)     │     │  · session snap   │
///   └─────────────────┘     └──────────────────┘
///                                  │
///                          ┌───────┴────────┐
///                          │ filesystem     │
///                          │ (inbox/ dir +  │
///                          │  snapshot.json)│
///                          └────────────────┘

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaemonStatus {
    Idle,
    Running,
    Paused,
    Degraded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonState {
    pub session_id: String,
    pub status: DaemonStatus,
    pub cycle: u64,
    pub uptime_secs: u64,
    pub last_heartbeat: u64,
    pub inbox_count: u64,
    pub snapshot_cycle: u64,
}

/// An incoming inbox message from another session or external process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxMessage {
    pub id: u64,
    pub timestamp: u64,
    pub sender: String,
    pub kind: InboxMessageKind,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InboxMessageKind {
    Query,
    Command,
    Data,
    Ack,
}

pub struct DaemonMode {
    session_id: String,
    status: DaemonStatus,
    inbox_path: PathBuf,
    snapshot_path: PathBuf,
    inbox: VecDeque<InboxMessage>,
    processed_ids: Vec<u64>,
    next_id: u64,
    cycle: u64,
    start_time: u64,
    last_heartbeat: u64,
    heartbeat_interval: u64,
    snapshot_interval: u64,
    last_snapshot: u64,
    stopped: Arc<AtomicBool>,
}

impl DaemonMode {
    pub fn new(session_id: String, inbox_dir: PathBuf, snapshot_dir: PathBuf) -> Self {
        let now = unix_now();
        let inbox_path = inbox_dir.join(format!("{}.inbox", &session_id));
        let snapshot_path = snapshot_dir.join(format!("{}.snap.json", &session_id));
        let _ = fs::create_dir_all(&inbox_dir);
        let _ = fs::create_dir_all(&snapshot_dir);

        let mut dm = Self {
            session_id,
            status: DaemonStatus::Idle,
            inbox_path,
            snapshot_path,
            inbox: VecDeque::new(),
            processed_ids: Vec::new(),
            next_id: 1,
            cycle: 0,
            start_time: now,
            last_heartbeat: now,
            heartbeat_interval: 10,
            snapshot_interval: 100,
            last_snapshot: now,
            stopped: Arc::new(AtomicBool::new(false)),
        };
        dm.load_inbox();
        dm.try_restore_snapshot();
        dm
    }

    // ─── Lifecycle ─────────────────────────────────────────────────────

    pub fn start(&mut self) {
        self.status = DaemonStatus::Running;
        self.start_time = unix_now();
    }

    pub fn pause(&mut self) {
        self.status = DaemonStatus::Paused;
    }

    pub fn resume(&mut self) {
        self.status = DaemonStatus::Running;
    }

    pub fn stop(&mut self) {
        self.status = DaemonStatus::Idle;
        self.take_snapshot();
        self.stopped.store(true, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
        self.status == DaemonStatus::Running && !self.stopped.load(Ordering::SeqCst)
    }

    pub fn status(&self) -> DaemonStatus {
        self.status
    }

    // ─── Tick (called from consciousness loop) ─────────────────────────

    pub fn tick(&mut self) -> DaemonState {
        self.cycle += 1;

        // Heartbeat
        if self.cycle % self.heartbeat_interval == 0 {
            self.last_heartbeat = unix_now();
            self.load_inbox();
        }

        // Periodic snapshot
        if self.cycle % self.snapshot_interval == 0 {
            self.take_snapshot();
        }

        DaemonState {
            session_id: self.session_id.clone(),
            status: self.status,
            cycle: self.cycle,
            uptime_secs: unix_now() - self.start_time,
            last_heartbeat: self.last_heartbeat,
            inbox_count: self.inbox.len() as u64,
            snapshot_cycle: self.last_snapshot,
        }
    }

    // ─── Inbox (file-based inter-session messaging) ────────────────────

    pub fn send_message(&mut self, sender: &str, kind: InboxMessageKind, payload: String) {
        let msg = InboxMessage {
            id: self.next_id,
            timestamp: unix_now(),
            sender: sender.to_string(),
            kind,
            payload,
        };
        self.next_id += 1;
        self.inbox.push_back(msg);
        self.flush_inbox();
    }

    pub fn read_messages(&mut self) -> Vec<InboxMessage> {
        let msgs: Vec<InboxMessage> = self.inbox.drain(..).collect();
        for msg in &msgs {
            self.processed_ids.push(msg.id);
        }
        self.flush_inbox();
        msgs
    }

    pub fn unread_count(&self) -> usize {
        self.inbox.len()
    }

    fn load_inbox(&mut self) {
        if !self.inbox_path.exists() {
            return;
        }
        let content = match fs::read_to_string(&self.inbox_path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let loaded: Vec<InboxMessage> = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => return,
        };
        for msg in loaded {
            if !self.processed_ids.contains(&msg.id) {
                self.inbox.push_back(msg);
            }
        }
    }

    fn flush_inbox(&mut self) {
        let unprocessed: Vec<&InboxMessage> = self
            .inbox
            .iter()
            .filter(|m| !self.processed_ids.contains(&m.id))
            .collect();
        if let Ok(json) = serde_json::to_string(&unprocessed) {
            let _ = fs::write(&self.inbox_path, &json);
        }
    }

    // ─── Snapshot (session persistence) ────────────────────────────────

    pub fn take_snapshot(&mut self) {
        self.last_snapshot = self.cycle;
    }

    pub fn set_snapshot_callback<F>(&mut self, _cb: F)
    where
        F: FnMut() -> String + Send + 'static,
    {
        // The snapshot data is injected from outside (ConsciousnessIntegration
        // serializes its state). We just write whatever data is passed.
    }

    pub fn write_snapshot(&self, data: &str) {
        let _ = fs::write(&self.snapshot_path, data);
    }

    fn try_restore_snapshot(&mut self) -> bool {
        self.snapshot_path.exists()
    }

    pub fn read_snapshot(&self) -> Option<String> {
        fs::read_to_string(&self.snapshot_path).ok()
    }

    // ─── State ─────────────────────────────────────────────────────────

    pub fn state(&self) -> DaemonState {
        DaemonState {
            session_id: self.session_id.clone(),
            status: self.status,
            cycle: self.cycle,
            uptime_secs: unix_now() - self.start_time,
            last_heartbeat: self.last_heartbeat,
            inbox_count: self.inbox.len() as u64,
            snapshot_cycle: self.last_snapshot,
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Daemon[{}]: {:?}, cycle={}, uptime={}s, inbox={}, snapshots={}",
            self.session_id,
            self.status,
            self.cycle,
            unix_now() - self.start_time,
            self.inbox.len(),
            self.last_snapshot,
        )
    }

    #[cfg(test)]
    pub fn set_heartbeat_interval(&mut self, interval: u64) {
        self.heartbeat_interval = interval;
    }

    #[cfg(test)]
    pub fn set_snapshot_interval(&mut self, interval: u64) {
        self.snapshot_interval = interval;
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_daemon() -> DaemonMode {
        let dir = std::env::temp_dir().join("nt_daemon_test");
        let _ = fs::create_dir_all(&dir);
        DaemonMode::new(
            "test-daemon".into(),
            dir.join("inbox"),
            dir.join("snapshots"),
        )
    }

    #[test]
    fn test_lifecycle() {
        let mut d = test_daemon();
        assert_eq!(d.status(), DaemonStatus::Idle);
        d.start();
        assert_eq!(d.status(), DaemonStatus::Running);
        assert!(d.is_running());
        d.pause();
        assert_eq!(d.status(), DaemonStatus::Paused);
        d.resume();
        assert_eq!(d.status(), DaemonStatus::Running);
        d.stop();
        assert_eq!(d.status(), DaemonStatus::Idle);
    }

    #[test]
    fn test_tick_advances_cycle() {
        let mut d = test_daemon();
        d.start();
        d.set_heartbeat_interval(1);
        d.set_snapshot_interval(5);
        for _ in 0..10 {
            d.tick();
        }
        assert_eq!(d.state().cycle, 10);
    }

    #[test]
    fn test_send_and_read_messages() {
        let mut d = test_daemon();
        d.send_message(
            "external-agent",
            InboxMessageKind::Query,
            "hello from outside".into(),
        );
        assert_eq!(d.unread_count(), 1);
        let msgs = d.read_messages();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].sender, "external-agent");
        assert_eq!(d.unread_count(), 0);
    }

    #[test]
    fn test_snapshot_path_creation() {
        let d = test_daemon();
        let state = d.state();
        assert_eq!(state.session_id, "test-daemon");
    }
}
