//! skill_scan — 从 `nt_mind_skill_engine.rs` 拆分 (行为零变更).
//! 目录扫描 + 解析 + 索引: `load_all` / `sync_to_kb_index` / `build_index` / discover 家族.

use std::path::{Path, PathBuf};

use super::DiscoveredSkill;
use super::SkillDocEntry;
use super::SkillEngine;
use super::SkillQualityScorer;
use super::evomal_poison_scan;
use crate::l5_cognition::l1_facade::{SkillRecord, skill_upsert};

impl SkillEngine {
    /// Scan the skills directory and load all valid skill files.
    pub fn load_all(&mut self) -> Vec<SkillDocEntry> {
        self.skills.clear();
        self.trigger_index.clear();
        self.e8_index.clear();

        let dir = &self.skills_dir;
        if !dir.exists() {
            let _ = std::fs::create_dir_all(dir);
            return Vec::new();
        }

        let mut loaded = Vec::new();
        self.quality_stats.clear();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Some(skill) = SkillDocEntry::from_file(&skill_md) {
                            let scores = SkillQualityScorer::evaluate(&skill);
                            // P6 SkillTrustBench 安全门 (Tencent AIG absorbed, R-P79):
                            // 静态 T01-T09 扫描 — 命中任一攻击分类即拒收, 不进入生产索引。
                            let (trust_findings, trust_verdict) =
                                crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::scan_skill_content(&skill.content);
                            let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::InspectionResult::Allow);
                            // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                            // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                            let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                            // A5 安全门 (SkillNet absorb, R-P79): 含危险命令
                            // (rm -rf 等) 的技能拒收, 不进入生产检索索引。
                            if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                                self.quality_stats.insert(skill.name.clone(), scores);
                                loaded.push(skill);
                            } else if trust_rejected {
                                log::warn!(
                                    "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                    skill.name,
                                    trust_findings.len(),
                                    trust_findings.first().map(|f| f.id).unwrap_or("?")
                                );
                            } else if !poison_ok {
                                log::warn!(
                                    "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                    skill.name
                                );
                            }
                        }
                    }
                    continue;
                }
                if path.extension().is_some_and(|e| e == "md") {
                    if let Some(skill) = SkillDocEntry::from_file(&path) {
                        let scores = SkillQualityScorer::evaluate(&skill);
                        // P6 SkillTrustBench 安全门 (同目录型技能, R-P79)。
                        let (trust_findings, trust_verdict) =
                            crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::scan_skill_content(&skill.content);
                        let trust_rejected = !matches!(trust_verdict, crate::l3_embodiment::nt_shield::shield_core::tool_inspection_stack::InspectionResult::Allow);
                        // E6 防护层硬化 (src9 EVOMAL 毒化扫描): 折入 R-P108
                        // 五维门 — 命中毒化模式即拒收, 阻断 promote。Err 保守视为拒收。
                        let poison_ok = evomal_poison_scan(&skill).unwrap_or(false);
                        if scores.safety >= 0.8 && !trust_rejected && poison_ok {
                            self.quality_stats.insert(skill.name.clone(), scores);
                            loaded.push(skill);
                        } else if trust_rejected {
                            log::warn!(
                                "SkillTrustBench 拒收技能 `{}` ({} 命中): {}",
                                skill.name,
                                trust_findings.len(),
                                trust_findings.first().map(|f| f.id).unwrap_or("?")
                            );
                        } else if !poison_ok {
                            log::warn!(
                                "EVOMAL 毒化扫描拒收技能 `{}` (src9 模式命中)",
                                skill.name
                            );
                        }
                    }
                }
            }
        }

        self.skills = loaded;
        self.build_index();
        // Phase 4 库治理: load 时自动去重 (T3 生产接线, Dark Forest; retire/rebalance 见 maintain())
        self.prune_semantic_duplicates();
        // UCN Phase 1 写通: 若挂接 KB, 扫描后自动把索引同步进 skills_index 表。
        if let Some(kb) = self.kb.clone() {
            if let Ok(conn) = kb.raw_conn() {
                let _ = self.sync_to_kb_index(&conn);
            }
        }
        self.skills.clone()
    }

    /// 把当前内存索引同步到 KB `skills_index` 表 (UCN Phase 1 写通)。
    /// 返回本次真正写入/更新的条数; 内容未变化 (content_hash 相同) 被去重跳过。
    pub fn sync_to_kb_index(&self, conn: &rusqlite::Connection) -> Result<usize, String> {
        use crate::l5_cognition::l1_facade::skill_content_hash;
        use std::collections::HashSet;

        let mut written = 0usize;
        let mut seen: HashSet<String> = HashSet::new();
        for skill in &self.skills {
            if !seen.insert(skill.name.clone()) {
                continue;
            }
            let record = SkillRecord {
                id: uuid::Uuid::new_v4().to_string(),
                name: skill.name.clone(),
                description: Some(skill.description.clone()),
                source_path: Some(skill.path.to_string_lossy().to_string()),
                tags: if skill.triggers.is_empty() {
                    None
                } else {
                    Some(skill.triggers.join(","))
                },
                is_builtin: false,
                last_indexed_at: Some(crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now()),
                created_at: crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now(),
                updated_at: crate::l4_emotion::nt_memory::nt_memory_kb::nt_memory_unify::now(),
                content_hash: Some(skill_content_hash(&skill.content)),
            };
            if skill_upsert(conn, &record.name, &record)? {
                written += 1;
            }
        }
        Ok(written)
    }

    /// Build trigger and E8 mode indices.
    pub(crate) fn build_index(&mut self) {
        self.trigger_index.clear();
        self.e8_index.clear();

        for (i, skill) in self.skills.iter().enumerate() {
            for trigger in &skill.triggers {
                let key = trigger.to_lowercase();
                self.trigger_index.entry(key).or_default().push(i);
            }
            for mode in &skill.e8_modes {
                self.e8_index.entry(*mode).or_default().push(i);
            }
        }
    }

    /// Find all skill files in the workspace and agent directories.
    /// Legacy compatibility: discovers but does NOT load into this engine.
    pub fn discover_skills() -> Vec<DiscoveredSkill> {
        let mut skills = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        // 1. ~/.neotrix/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".neotrix").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 2. ~/.agents/skills/
        if let Ok(home) = std::env::var("HOME") {
            let dir = PathBuf::from(&home).join(".agents").join("skills");
            if dir.exists() {
                Self::scan_discover_dir(&dir, &mut seen, &mut skills);
            }
        }

        // 3. Workspace skills/
        let ws = Path::new("skills");
        if ws.exists() {
            Self::scan_discover_dir(ws, &mut seen, &mut skills);
        }

        skills
    }

    fn scan_discover_dir(dir: &Path, seen: &mut Vec<String>, skills: &mut Vec<DiscoveredSkill>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    if seen.contains(&name) { continue; }
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        let content = std::fs::read_to_string(&skill_md).unwrap_or_default();
                        let description = Self::extract_frontmatter_desc(&content);
                        seen.push(name.clone());
                        skills.push(DiscoveredSkill { name, description, path: skill_md });
                    }
                }
            }
        }
    }

    fn extract_frontmatter_desc(content: &str) -> String {
        let stripped = content.trim_start();
        if !stripped.starts_with("---") { return String::new(); }
        if let Some(end) = stripped[3..].find("---") {
            let frontmatter = &stripped[3..3 + end];
            for line in frontmatter.lines() {
                if let Some(val) = line.trim().strip_prefix("description:") {
                    return val.trim().to_string();
                }
            }
        }
        String::new()
    }

    /// Find all SKILL.md files recursively within a directory (legacy compat).
    pub fn find_skill_mds(dir: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();
        if dir.is_file() && dir.ends_with("SKILL.md") {
            results.push(dir.to_path_buf());
            return results;
        }
        Self::find_skill_mds_recursive(dir, &mut results);
        results
    }

    fn find_skill_mds_recursive(dir: &Path, results: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let fname = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();
                    if fname.starts_with('.') || fname == "node_modules" || fname == "target" {
                        continue;
                    }
                    Self::find_skill_mds_recursive(&path, results);
                } else if path.ends_with("SKILL.md") {
                    results.push(path);
                }
            }
        }
    }
}
