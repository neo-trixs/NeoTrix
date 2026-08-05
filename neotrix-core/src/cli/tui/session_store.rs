#[derive(Debug, Clone)]
pub struct SessionStorage {
    sessions: Vec<String>,
}

impl Default for SessionStorage {
    fn default() -> Self { Self::new() }
}

impl SessionStorage {
    pub fn new() -> Self { Self { sessions: Vec::new() } }

    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.clone()
    }

    pub fn add_session(&mut self, _name: &str) {
        self.sessions.push(_name.to_string());
    }

    pub fn remove_session(&mut self, _name: &str) {
        self.sessions.retain(|s| s != _name);
    }

    pub fn contains(&self, _name: &str) -> bool {
        self.sessions.contains(&_name.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SessionData {
    pub id: String,
    pub name: String,
    pub messages: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 会话存储 — 真实落盘到 KB `session_logs` 表 + `~/.neotrix/session-logs/*.md`
/// 蒸馏输入。覆盖 `/session` 全子命令，替代原内存桩 (R-P79 接线)。
pub struct SessionStore {
    kb: crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase,
    base: std::path::PathBuf,
}

impl Default for SessionStore {
    fn default() -> Self { Self::new() }
}

impl SessionStore {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let base = std::path::PathBuf::from(home).join(".neotrix");
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(None)
            .unwrap_or_else(|_| {
                crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(base.join("knowledge.db")))
                    .expect("KB open fallback")
            });
        Self { kb, base }
    }

    fn logs_dir(&self) -> std::path::PathBuf {
        self.base.join("session-logs")
    }

    pub fn list_sessions(&self) -> Vec<SessionData> {
        let list = self.kb.session_log_list();
        match list {
            Ok(rows) => rows
                .into_iter()
                .map(|(session_id, _, _)| self.load_session(&session_id).unwrap_or(SessionData {
                    id: session_id.clone(),
                    name: session_id,
                    messages: vec![],
                    created_at: String::new(),
                    updated_at: String::new(),
                }))
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    pub fn save_session(&mut self, _name: &str, data: &SessionData) -> Result<(), String> {
        let meta = serde_json::json!({ "name": data.name, "created_at": data.created_at, "updated_at": data.updated_at });
        self.kb.session_log_append(&data.name, "", "session_header", Some(&meta))?;
        for msg in &data.messages {
            self.kb.session_log_append(&data.name, msg, "message", Some(&meta))?;
        }
        // 同步写 session-logs/*.md 供 SessionDistiller 蒸馏
        std::fs::create_dir_all(self.logs_dir()).map_err(|e| format!("mkdir session-logs: {}", e))?;
        let md_path = self.logs_dir().join(format!("{}.md", data.name));
        let mut content = format!("# Session {}\n\n", data.name);
        content.push_str(&format!("> created: {}\n> updated: {}\n\n", data.created_at, data.updated_at));
        for msg in &data.messages {
            content.push_str(msg);
            content.push('\n');
        }
        std::fs::write(&md_path, content).map_err(|e| format!("write session log: {}", e))?;
        Ok(())
    }

    pub fn load_session(&self, _name: &str) -> Result<SessionData, String> {
        let entries = self.kb.session_log_get(_name, 500, 0)?;
        if entries.is_empty() {
            return Err(format!("会话 '{}' 不存在", _name));
        }
        let mut messages = Vec::new();
        let mut created_at = String::new();
        let mut updated_at = String::new();
        for (_, content, _ctype, ts, meta) in entries.iter().rev() {
            if let Some(m) = meta {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(m) {
                    if created_at.is_empty() {
                        created_at = v.get("created_at").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    }
                    updated_at = v.get("updated_at").and_then(|x| x.as_str()).unwrap_or(ts).to_string();
                }
            }
            if !content.is_empty() {
                messages.push(content.clone());
            }
        }
        Ok(SessionData {
            id: _name.to_string(),
            name: _name.to_string(),
            messages,
            created_at,
            updated_at,
        })
    }

    pub fn delete_session(&mut self, _name: &str) -> Result<(), String> {
        self.kb.session_log_delete(_name)?;
        let md_path = self.logs_dir().join(format!("{}.md", _name));
        if md_path.exists() {
            let _ = std::fs::remove_file(&md_path);
        }
        Ok(())
    }

    pub fn fork(&mut self, _name: &str) -> Result<String, String> {
        let data = self.load_session(_name)?;
        let new_name = format!("{}-fork-{}", _name, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs());
        let mut fork_data = data.clone();
        fork_data.name = new_name.clone();
        fork_data.created_at = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        fork_data.updated_at = fork_data.created_at.clone();
        self.save_session(&new_name, &fork_data)?;
        Ok(new_name)
    }

    pub fn export_to_file(&self, _name: &str, _path: &str) -> Result<(), String> {
        let json = self.export_to_json(_name)?;
        std::fs::write(_path, json).map_err(|e| format!("export to file: {}", e))
    }

    pub fn export_to_json(&self, _name: &str) -> Result<String, String> {
        let data = self.load_session(_name)?;
        serde_json::to_string_pretty(&serde_json::json!({
            "name": data.name,
            "id": data.id,
            "created_at": data.created_at,
            "updated_at": data.updated_at,
            "messages": data.messages,
        })).map_err(|e| format!("serialize session: {}", e))
    }

    pub fn import_from_file(&mut self, _path: &str) -> Result<String, String> {
        let raw = std::fs::read_to_string(_path).map_err(|e| format!("read import file: {}", e))?;
        let v: serde_json::Value = serde_json::from_str(&raw).map_err(|e| format!("parse import: {}", e))?;
        let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("imported").to_string();
        let messages = v.get("messages")
            .and_then(|m| m.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect::<Vec<_>>())
            .unwrap_or_default();
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let data = SessionData {
            id: name.clone(),
            name: name.clone(),
            messages,
            created_at: now.clone(),
            updated_at: now,
        };
        self.save_session(&name, &data)?;
        Ok(name)
    }

    pub fn get_last_session(&self) -> Option<String> {
        let list = self.kb.session_log_list();
        list.ok().and_then(|rows| rows.into_iter().next().map(|(id, _, _)| id))
    }

    /// 触发会话蒸馏: 从 `~/.neotrix/session-logs/` 提取行为模式并产出报告
    pub fn distill(&mut self) -> Result<crate::neotrix::nt_mind_distiller::DistillationReport, String> {
        let mut d = crate::neotrix::nt_mind_distiller::SessionDistiller::with_paths(
            self.logs_dir(),
            self.base.join("AGENTS-distilled.md"),
        );
        d.generate_distillation_report();
        Ok(d.generate_distillation_report())
    }
}
