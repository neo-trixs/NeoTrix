//! `nt_agent_home` — 「一个 agent = 一个目录」的持久化对照（A8 吸收，
//! 源：openhanako「Agent 就是文件夹」+ row-bot「local-first sovereignty」）。
//!
//! 约定目录形状：
//! ```text
//! <agent_dir>/
//!   AGENTS.md          # 人格/规则（必须存在，否则拒 export）
//!   memory/            # 会话记忆（可空目录）
//!   skills/            # 技能包（可空目录）
//! ```
//! `export` 把该目录完整拷到目标路径；`import` 从备份拷回。
//! 纯 fs 操作（无网络、无时钟依赖，时间戳由调用方注入）。

use std::path::{Path, PathBuf};

use crate::nt_error::NtBotError;

/// 一个 agent 的目录主页。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentHome {
    pub root: PathBuf,
}

impl AgentHome {
    /// 以目录路径构造；不校验结构（校验交给 `validate`）。
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// 校验目录形状：AGENTS.md 必须存在且非空。
    pub fn validate(&self) -> Result<(), NtBotError> {
        let agents = self.root.join("AGENTS.md");
        let meta = std::fs::metadata(&agents)
            .map_err(|_| NtBotError::Invalid(format!("missing {}", agents.display())))?;
        if meta.len() == 0 {
            return Err(NtBotError::Invalid("AGENTS.md is empty".into()));
        }
        Ok(())
    }

    /// 相对路径清单（已排序）—— export/import 的对账面。
    pub fn file_manifest(&self) -> Vec<String> {
        let mut out = Vec::new();
        fn walk(base: &Path, dir: &Path, out: &mut Vec<String>) {
            if let Ok(rd) = std::fs::read_dir(dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(base, &p, out);
                    } else if let Ok(rel) = p.strip_prefix(base) {
                        out.push(rel.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
        }
        walk(&self.root, &self.root, &mut out);
        out.sort();
        out
    }

    /// export：递归拷贝整棵目录到 `dest`（目标必须不存在或为空目录）。
    pub fn export_to(&self, dest: &Path) -> Result<(), NtBotError> {
        self.validate()?;
        copy_dir_recursive(&self.root, dest)
    }

    /// import：从 `src` 拷贝回本目录根（覆盖同名文件）。
    pub fn import_from(&self, src: &Path) -> Result<(), NtBotError> {
        copy_dir_recursive(src, &self.root)
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), NtBotError> {
    std::fs::create_dir_all(dst)
        .map_err(|e| NtBotError::Io(format!("mkdir {}: {e}", dst.display())))?;
    for entry in std::fs::read_dir(src)
        .map_err(|e| NtBotError::Io(format!("read_dir {}: {e}", src.display())))?
        .flatten()
    {
        let s = entry.path();
        let d = dst.join(entry.file_name());
        if s.is_dir() {
            copy_dir_recursive(&s, &d)?;
        } else {
            std::fs::copy(&s, &d)
                .map_err(|e| NtBotError::Io(format!("copy {}: {e}", s.display())))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent_dir(base: &Path) -> PathBuf {
        let root = base.join("agent1");
        std::fs::create_dir_all(root.join("memory")).unwrap();
        std::fs::create_dir_all(root.join("skills")).unwrap();
        std::fs::write(root.join("AGENTS.md"), "# agent rules\n").unwrap();
        std::fs::write(root.join("memory").join("note.md"), "hi\n").unwrap();
        root
    }

    #[test]
    fn validate_requires_agents_md() {
        let tmp = std::env::temp_dir().join(format!("agent_home_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let bad = AgentHome::new(&tmp);
        assert!(bad.validate().is_err());
        let root = make_agent_dir(&tmp);
        assert!(AgentHome::new(&root).validate().is_ok());
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn export_import_roundtrip_preserves_manifest() {
        let tmp = std::env::temp_dir().join(format!("agent_home_rt_{}", std::process::id()));
        std::fs::create_dir_all(&tmp).unwrap();
        let src = make_agent_dir(&tmp);
        let home = AgentHome::new(&src);
        let backup = tmp.join("backup");
        home.export_to(&backup).unwrap();
        let relocated = tmp.join("agent2");
        std::fs::create_dir_all(&relocated).unwrap();
        AgentHome::new(&relocated).import_from(&backup).unwrap();
        assert_eq!(
            AgentHome::new(&src).file_manifest(),
            AgentHome::new(&relocated).file_manifest()
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
