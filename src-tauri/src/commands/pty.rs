//! PTY 终端模块 — portable-pty 驱动的终端会话管理

use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::Mutex;
use tokio::sync::mpsc;
use serde::Serialize;
use portable_pty::{PtySize, native_pty_system, CommandBuilder, ChildKiller, PtyPair};

/// PTY 事件（流式输出到前端）
#[derive(Debug, Clone, Serialize)]
pub struct PtyEvent {
    pub session_id: String,
    pub event_type: PtyEventType,
    pub data: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum PtyEventType {
    Output,
    Exit(i32),
}

/// PTY 会话
struct PtySession {
    pair: PtyPair,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

/// PTY 管理器
pub struct PtyManager {
    sessions: Mutex<HashMap<String, PtySession>>,
    sender: mpsc::UnboundedSender<PtyEvent>,
}

impl PtyManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<PtyEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (Self { sessions: Mutex::new(HashMap::new()), sender: tx }, rx)
    }

    pub fn spawn(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let system = native_pty_system();
        let pair = system.openpty(PtySize {
            rows, cols,
            pixel_width: 0, pixel_height: 0,
        }).map_err(|e| format!("openpty failed: {}", e))?;

        let cmd = if cfg!(target_os = "windows") {
            CommandBuilder::new("powershell.exe")
        } else {
            let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".to_string());
            CommandBuilder::new(shell)
        };

        let child = pair.slave.spawn_command(cmd)
            .map_err(|e| format!("spawn failed: {}", e))?;
        let killer = child.clone_killer();
        let mut reader = pair.master.try_clone_reader()
            .map_err(|e| format!("clone reader failed: {}", e))?;
        let writer = pair.master.take_writer()
            .map_err(|e| format!("take writer failed: {}", e))?;

        let sid = session_id.to_string();
        let tx = self.sender.clone();

        // 后台读取线程: PTY → mpsc channel
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => {
                        let _ = tx.send(PtyEvent {
                            session_id: sid.clone(),
                            event_type: PtyEventType::Exit(0),
                            data: String::new(),
                        });
                        break;
                    }
                    Ok(n) => {
                        let data = String::from_utf8_lossy(&buf[..n]).to_string();
                        if tx.send(PtyEvent {
                            session_id: sid.clone(),
                            event_type: PtyEventType::Output,
                            data,
                        }).is_err() { break; }
                    }
                }
            }
        });

        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(session_id.to_string(), PtySession {
            pair,
            writer: Box::new(writer),
            killer,
        });

        Ok(())
    }

    pub fn write(&self, session_id: &str, data: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session {} not found", session_id))?;
        session.writer.write_all(data.as_bytes()).map_err(|e| format!("write failed: {}", e))?;
        session.writer.flush().map_err(|e| format!("flush failed: {}", e))?;
        Ok(())
    }

    pub fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions.get_mut(session_id).ok_or_else(|| format!("Session {} not found", session_id))?;
        session.pair.master.resize(PtySize {
            rows, cols, pixel_width: 0, pixel_height: 0,
        }).map_err(|e| format!("resize failed: {}", e))?;
        Ok(())
    }

    pub fn close(&self, session_id: &str) {
        let mut sessions = self.sessions.lock().unwrap_or_else(|e| { log::warn!("PTY sessions mutex poisoned"); e.into_inner() });
        if let Some(mut session) = sessions.remove(session_id) {
            let _ = session.killer.kill();
        }
    }

}

// ========== Tauri Command Wrappers ==========

use tauri::State;
use std::sync::Arc;

#[tauri::command]
pub fn pty_spawn(
    session_id: String,
    cols: u16,
    rows: u16,
    manager: State<'_, Arc<PtyManager>>,
) -> Result<(), String> {
    manager.spawn(&session_id, cols, rows)
}

#[tauri::command]
pub fn pty_write(
    session_id: String,
    data: String,
    manager: State<'_, Arc<PtyManager>>,
) -> Result<(), String> {
    manager.write(&session_id, &data)
}

#[tauri::command]
pub fn pty_resize(
    session_id: String,
    cols: u16,
    rows: u16,
    manager: State<'_, Arc<PtyManager>>,
) -> Result<(), String> {
    manager.resize(&session_id, cols, rows)
}

#[tauri::command]
pub fn pty_close(
    session_id: String,
    manager: State<'_, Arc<PtyManager>>,
) -> Result<(), String> {
    manager.close(&session_id);
    Ok(())
}
