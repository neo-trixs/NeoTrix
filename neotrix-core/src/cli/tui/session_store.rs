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
        Self::with_base(std::path::PathBuf::from(home).join(".neotrix"))
    }

    /// 测试/隔离环境: 指定 base 目录 (KB + session-logs 均在其下)
    pub fn with_base(base: std::path::PathBuf) -> Self {
        let kb = crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase::open(Some(base.join("knowledge.db")))
            .expect("KB open");
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
        // P1-4 修复: 先删除旧记录再重写 → 幂等 (重复 save 不累积双份消息)。
        let _ = self.kb.session_log_delete(_name);
        let meta = serde_json::json!({ "name": data.name, "created_at": data.created_at, "updated_at": data.updated_at });
        self.kb.session_log_append(&data.name, "", "session_header", Some(&meta))?;
        for msg in &data.messages {
            self.kb.session_log_append(&data.name, msg, "message", Some(&meta))?;
        }
        // 同步写 session-logs/*.md 供 SessionDistiller 蒸馏
        std::fs::create_dir_all(self.logs_dir()).map_err(|e| format!("mkdir session-logs: {}", e))?;
        // P1-4 修复: 文件名消毒 — 只保留安全字符, 防 `../` 路径穿越写出 session-logs 目录。
        let safe_name: String = _name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .trim_start_matches('_')
            .chars()
            .take(80)
            .collect();
        let md_path = self.logs_dir().join(format!("{}.md", safe_name));
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
        let safe_name: String = _name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let md_path = self.logs_dir().join(format!("{}.md", safe_name));
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
        Ok(d.generate_distillation_report())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 端到端蒸馏链路验证 (R-P21 双重验证: 编译期调用链 + 运行时数据落地):
    /// save_session → KB session_logs + session-logs/*.md 落盘 → distill() 读取 md → 产出报告
    #[test]
    fn test_distill_end_to_end() {
        let dir = std::env::temp_dir().join(format!("nt_session_e2e_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        let mut store = SessionStore::with_base(dir.clone());

        let data = SessionData {
            id: "e2e-1".to_string(),
            name: "e2e-1".to_string(),
            messages: vec![
                "[2026-08-05][Coding] 用户要求同步执行并行任务 (reward=1.00, success=true)".to_string(),
                "[2026-08-05][Coding] 还有需要进化的路线吗 (reward=0.80, success=true)".to_string(),
            ],
            created_at: "2026-08-05 10:00:00".to_string(),
            updated_at: "2026-08-05 10:05:00".to_string(),
        };

        // 1. save → KB + md 文件
        store.save_session("e2e-1", &data).expect("save session");
        let md_path = dir.join("session-logs").join("e2e-1.md");
        assert!(md_path.exists(), "session-logs/e2e-1.md 必须落盘");
        let md_content = std::fs::read_to_string(&md_path).expect("read md");
        assert!(md_content.contains("同步执行"), "md 内容需保留消息文本");

        // 2. KB 可读回
        let loaded = store.load_session("e2e-1").expect("load session");
        assert_eq!(loaded.name, "e2e-1");
        assert_eq!(loaded.messages.len(), 2);

        // 3. distill → 从 md 提取行为模式
        let report = store.distill().expect("distill");
        assert!(report.session_count >= 1, "报告应覆盖已保存会话");
        assert!(
            report.patterns.iter().any(|p| p.name == "parallel_dispatch"),
            "应识别 parallel_dispatch 模式, got {:?}",
            report.patterns.iter().map(|p| p.name.clone()).collect::<Vec<_>>()
        );
        assert!(
            report.suggestions.iter().any(|s| s.contains("同步执行")),
            "应产出 actionable 建议"
        );

        // 4. delete → 双端清理
        store.delete_session("e2e-1").expect("delete session");
        assert!(!md_path.exists(), "delete 应同时移除 md 文件");

        let _ = std::fs::remove_dir_all(&dir);
    }
}
