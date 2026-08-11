//! Skills command — /skills list | install <source> | info <name> | scan | active | load <name> | unload <name>

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::RwLock;

use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_unify::skill_list_all;
use crate::neotrix::l3_memory_impl::nt_memory_kb::KnowledgeBase;
use crate::neotrix::l8_autonomic_impl::nt_mind_skill_engine::SkillEngine;
use crate::neotrix::nt_mind::SelfIteratingBrain;

/// 惰性 KB 单例 — UCN Phase 1 读通: skill list 优先从 skills_index 表读。
static KB: OnceLock<Arc<KnowledgeBase>> = OnceLock::new();

fn kb() -> Option<Arc<KnowledgeBase>> {
    let arc = KB.get_or_init(|| {
        Arc::new(KnowledgeBase::open(None).unwrap_or_else(|_| {
            // 兜底: 打开默认路径失败时退回临时库, 保持 CLI 可用
            KnowledgeBase::open(Some(std::env::temp_dir().join("neotrix-kb-fallback.db"))).expect("KB fallback open")
        }))
    });
    Some(Arc::clone(arc))
}

pub struct SkillCmd;

impl CliCommand for SkillCmd {
    fn name(&self) -> &str { "/skills" }
    fn aliases(&self) -> Vec<&str> { vec!["/skill"] }
    fn description(&self) -> &str {
        "Skills: /skills list | active | load <name> | unload <name> | install <source> | info <name> | scan"
    }
    fn execute(&self, args: &[String], _brain: Option<&Arc<RwLock<SelfIteratingBrain>>>) -> CommandOutput {
        let subcmd = args.first().map(|s| s.as_str()).unwrap_or("list");
        match subcmd {
            "list" => self.list_skills(),
            "active" => self.list_active(),
            "load" | "activate" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /skills load <skill-name>");
                }
                self.activate(name)
            }
            "unload" | "deactivate" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /skills unload <skill-name>");
                }
                self.deactivate(name)
            }
            "install" => {
                let source = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if source.is_empty() {
                    return CommandOutput::err("Usage: /skills install <path-to-skill.md-or-directory>");
                }
                self.install(source)
            }
            "info" => {
                let name = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if name.is_empty() {
                    return CommandOutput::err("Usage: /skills info <skill-name>");
                }
                self.info(name)
            }
            "scan" | "reload" => self.scan_skills(),
            "help" => CommandOutput::ok("Usage: /skills list | active | load <name> | unload <name> | install <source> | info <name> | scan"),
            _ => CommandOutput::err("Usage: /skills list | active | load <name> | unload <name> | install <source> | info <name> | scan"),
        }
    }
}

impl SkillCmd {
    fn skills_dir() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(&home).join(".neotrix").join("skills")
    }

    fn engine(&self) -> SkillEngine {
        let dir = Self::skills_dir();
        let mut engine = SkillEngine::new(dir);
        // UCN Phase 1 写通: 挂接 KB, load_all 末尾自动同步进 skills_index 表
        if let Some(kb) = kb() {
            engine = engine.with_kb(kb);
        }
        engine.load_all();
        engine
    }

    fn list_skills(&self) -> CommandOutput {
        // UCN Phase 1 读通: 优先从 KB skills_index 表读; 表空则回退文件扫描+写通
        if let Some(kb) = kb() {
            match kb.raw_conn() {
                Ok(conn) => match skill_list_all(&conn, 200) {
                    Ok(recs) if !recs.is_empty() => {
                        let mut output = format!("Found {} skills (KB index):\n\n", recs.len());
                        for (i, r) in recs.iter().enumerate() {
                            let desc = r.description.as_deref().unwrap_or("—");
                            output.push_str(&format!("{}. {} — {}\n", i + 1, r.name, desc));
                            if let Some(path) = &r.source_path {
                                output.push_str(&format!("     source: {}\n", path));
                            }
                            if let Some(tags) = &r.tags {
                                output.push_str(&format!("     tags: {}\n", tags));
                            }
                        }
                        output.push_str("\n(KB skills_index — /skills scan 增量刷新, /skills info <name> 查详情)");
                        return CommandOutput::ok(&output);
                    }
                    Ok(_) => {} // 表空 → 回退文件扫描 (下方)
                    Err(e) => {
                        return CommandOutput::err(&format!("KB skills_index 读取失败: {}", e));
                    }
                },
                Err(e) => {
                    return CommandOutput::err(&format!("KB 连接失败: {}", e));
                }
            }
        }
        // 回退: 文件系统扫描 (老路径)
        let engine = self.engine();
        let all = engine.list_all();
        if all.is_empty() {
            return CommandOutput::ok("No skills found. Use /skills install <path> to install one.");
        }
        let mut output = format!("Found {} skills (in {}):\n\n", all.len(), engine.skills_dir().display());
        for (i, s) in all.iter().enumerate() {
            let status = if s.active { "✅" } else { "  " };
            output.push_str(&format!("{}. {} [{}] {} — {}\n", i + 1, status, s.priority, s.name, s.description));
            if !s.triggers.is_empty() {
                output.push_str(&format!("     triggers: {}\n", s.triggers.join(", ")));
            }
            if !s.e8_modes.is_empty() {
                output.push_str(&format!("     e8_modes: {:?}\n", s.e8_modes));
            }
        }
        output.push_str("\nUse /skills info <name> for details, /skills load <name> to activate.");
        CommandOutput::ok(&output)
    }

    fn list_active(&self) -> CommandOutput {
        let engine = self.engine();
        let active = engine.list_active();
        if active.is_empty() {
            return CommandOutput::ok("No active skills. Use /skills load <name> to activate one.");
        }
        let mut output = format!("Active skills ({}):\n", active.len());
        for s in &active {
            output.push_str(&format!("  ✅ {} — {}\n", s.name, s.description));
        }
        CommandOutput::ok(&output)
    }

    fn activate(&self, name: &str) -> CommandOutput {
        let mut engine = self.engine();
        match engine.activate_skill(name) {
            Ok(()) => match engine.get_skill(name) {
                Some(skill) => CommandOutput::ok(&format!("✅ Skill '{}' activated — {}", name, skill.description)),
                None => CommandOutput::ok(&format!("✅ Skill '{}' activated", name)),
            },
            Err(e) => CommandOutput::err(&e),
        }
    }

    fn deactivate(&self, name: &str) -> CommandOutput {
        let mut engine = self.engine();
        match engine.deactivate_skill(name) {
            Ok(()) => CommandOutput::ok(&format!("Skill '{}' deactivated", name)),
            Err(e) => CommandOutput::err(&e),
        }
    }

    fn install(&self, source: &str) -> CommandOutput {
        let source_path = PathBuf::from(source);
        let dir = Self::skills_dir();
        if !dir.exists() && std::fs::create_dir_all(&dir).is_err() {
            return CommandOutput::err(&format!("Failed to create skills directory: {}", dir.display()));
        }

        let mut engine = self.engine();
        match engine.install_skill(&source_path) {
            Ok(()) => {
                let name = source_path.file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                CommandOutput::ok(&format!("✅ Skill installed to {}:\n  {}", dir.display(), name))
            }
            Err(e) => {
                // Fallback: try the legacy install approach for git URLs
                if source.starts_with("http://") || source.starts_with("https://")
                    || source.starts_with("git@") || source.starts_with("ssh://")
                {
                    return self.install_from_git(source);
                }
                CommandOutput::err(&format!("Install failed: {}", e))
            }
        }
    }

    fn install_from_git(&self, source: &str) -> CommandOutput {
        let tmp = std::env::temp_dir().join(format!("neotrix-skill-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);

        match std::process::Command::new("git")
            .args(["clone", "--depth", "1", source])
            .arg(&tmp)
            .output()
        {
            Ok(out) if out.status.success() => {
                let dir = Self::skills_dir();
                let mut engine = SkillEngine::new(dir);
                engine.load_all();
                let result = match engine.install_skill(&tmp) {
                    Ok(()) => CommandOutput::ok(&format!("✅ Skill installed from {}", source)),
                    Err(e) => CommandOutput::err(&format!("Install from git failed: {}", e)),
                };
                let _ = std::fs::remove_dir_all(&tmp);
                result
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                let _ = std::fs::remove_dir_all(&tmp);
                CommandOutput::err(&format!("Git clone failed: {}", stderr.trim()))
            }
            Err(e) => {
                let _ = std::fs::remove_dir_all(&tmp);
                CommandOutput::err(&format!("Git is not available: {}", e))
            }
        }
    }

    fn info(&self, name: &str) -> CommandOutput {
        let engine = self.engine();
        let skill = engine.get_skill(name);
        match skill {
            Some(s) => {
                let status = if s.active { "✅ active" } else { "inactive" };
                let mut output = format!("## Skill: {} ({})\n\n", s.name, status);
                output.push_str(&format!("**Description**: {}\n", s.description));
                output.push_str(&format!("**Path**: {}\n", s.path.display()));
                output.push_str(&format!("**Priority**: {}\n", s.priority));
                if !s.triggers.is_empty() {
                    output.push_str(&format!("**Triggers**: {}\n", s.triggers.join(", ")));
                }
                if !s.e8_modes.is_empty() {
                    output.push_str(&format!("**E8 Modes**: {:?}\n", s.e8_modes));
                }
                if !s.tools.is_empty() {
                    output.push_str(&format!("**Tools**: {}\n", s.tools.join(", ")));
                }
                if !s.hooks.is_empty() {
                    output.push_str(&format!("**Hooks**: {}\n", s.hooks.join(", ")));
                }
                let body = s.body().trim();
                if !body.is_empty() {
                    let preview: String = body.chars().take(500).collect();
                    output.push_str(&format!("\n**Body** (preview):\n{}\n", preview));
                    if body.chars().count() > 500 {
                        output.push_str("...\n");
                    }
                }
                CommandOutput::ok(&output)
            }
            None => CommandOutput::err(&format!("Skill '{}' not found. Use /skills list to see available skills.", name)),
        }
    }

    fn scan_skills(&self) -> CommandOutput {
        // UCN Phase 1 写通: 挂接 KB, load_all 自动同步进 skills_index 表
        let dir = Self::skills_dir();
        let mut engine = SkillEngine::new(dir.clone());
        if let Some(kb) = kb() {
            engine = engine.with_kb(kb);
        }
        let loaded = engine.load_all();
        let sync_note = if let Some(kb) = kb() {
            match kb.raw_conn() {
                Ok(conn) => match engine.sync_to_kb_index(&conn) {
                    Ok(n) => format!("\nKB skills_index 写通: {} 条新写入/更新", n),
                    Err(e) => format!("\nKB 写通失败: {}", e),
                },
                Err(e) => format!("\nKB 连接失败: {}", e),
            }
        } else {
            String::new()
        };
        let legacy = SkillEngine::discover_skills();
        let msg = if loaded.is_empty() && legacy.is_empty() {
            "No skills found. Use /skills install <path> to install one.".to_string()
        } else {
            let mut lines = format!("Scanned and loaded {} skill(s) from {}:", loaded.len(), dir.display());
            for s in &loaded {
                lines.push_str(&format!("\n  {} — {}", s.name, s.description));
            }
            if !legacy.is_empty() {
                lines.push_str(&format!("\n\nAlso discovered {} legacy skill(s) outside engine directory:", legacy.len()));
                for s in &legacy {
                    lines.push_str(&format!("\n  {} — {} (at {})", s.name, s.description, s.path.display()));
                }
            }
            lines
        };
        CommandOutput::ok(&format!("{}{}", msg, sync_note))
    }
}
